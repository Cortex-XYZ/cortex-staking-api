use crate::{
    client::{MonadClientError, MonadRpcClient},
    model::{
        AccountSummaryResponse, ApyResponse, EpochResponse, MonadDataQuery, MonadLocation,
        MonadNetwork, PendingRewardsResponse, StakedBalanceResponse, ValidatorMetadataResponse,
        WalletBalanceResponse, WithdrawalRequestResponse,
    },
};

#[derive(Debug, Clone)]
pub enum MonadDataError {
    InvalidWalletAddress,
    MissingWalletAddress,
    MissingValidatorId,
    MissingWithdrawId,
    RpcError,
    InvalidRpcResponse,
}

fn resolve_network(query: &MonadDataQuery) -> MonadNetwork {
    query.network.clone().unwrap_or_default()
}

fn resolve_location(query: &MonadDataQuery) -> MonadLocation {
    query.location.clone().unwrap_or_default()
}

fn require_wallet_address(query: &MonadDataQuery) -> Result<String, MonadDataError> {
    query
        .wallet_address
        .clone()
        .filter(|value| !value.is_empty())
        .ok_or(MonadDataError::MissingWalletAddress)
}

fn require_validator_id(query: &MonadDataQuery) -> Result<u64, MonadDataError> {
    query.validator_id.ok_or(MonadDataError::MissingValidatorId)
}

fn require_withdraw_id(query: &MonadDataQuery) -> Result<u8, MonadDataError> {
    query.withdraw_id.ok_or(MonadDataError::MissingWithdrawId)
}

fn map_client_error(error: MonadClientError) -> MonadDataError {
    tracing::error!(error = ?error, "Monad RPC request failed");

    match error {
        MonadClientError::InvalidResponse(_) => {
            MonadDataError::InvalidRpcResponse
        }

        MonadClientError::RpcRequestFailed(_)
        | MonadClientError::HttpError { .. }
        | MonadClientError::RpcError { .. } => {
            MonadDataError::RpcError
        }
    }
}

fn strip_0x(value: &str) -> &str {
    value.strip_prefix("0x").unwrap_or(value)
}

fn hex_to_u128(value: &str) -> Result<u128, MonadDataError> {
    u128::from_str_radix(strip_0x(value), 16).map_err(|_| MonadDataError::InvalidRpcResponse)
}

fn wei_to_mon_string(wei: u128) -> String {
    let whole = wei / 1_000_000_000_000_000_000u128;
    let fractional = wei % 1_000_000_000_000_000_000u128;

    if fractional == 0 {
        return whole.to_string();
    }

    let mut fractional_str = format!("{:018}", fractional);

    while fractional_str.ends_with('0') {
        fractional_str.pop();
    }

    format!("{}.{}", whole, fractional_str)
}

fn read_word(hex: &str, index: usize) -> Result<&str, MonadDataError> {
    let clean = strip_0x(hex);
    let start = index * 64;
    let end = start + 64;

    clean.get(start..end)
        .ok_or(MonadDataError::InvalidRpcResponse)
}

fn word_to_u64(word: &str) -> Result<u64, MonadDataError> {
    u64::from_str_radix(word, 16).map_err(|_| MonadDataError::InvalidRpcResponse)
}

fn word_to_u128(word: &str) -> Result<u128, MonadDataError> {
    u128::from_str_radix(word, 16)
        .map_err(|_| MonadDataError::InvalidRpcResponse)
}

fn word_to_bool(word: &str) -> Result<bool, MonadDataError> {
    Ok(word_to_u64(word)? != 0)
}

fn word_to_address(word: &str) -> Result<String, MonadDataError> {
    if word.len() != 64 || !word.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(MonadDataError::InvalidRpcResponse);
    }

    let address = &word[24..64];

    Ok(format!("0x{}", address.to_lowercase()))
}

fn encode_address(address: &str) -> Result<String, MonadDataError> {
    let clean = strip_0x(address);

    if clean.len() != 40 || !clean.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(MonadDataError::InvalidWalletAddress);
    }

    Ok(format!("{:0>64}", clean.to_lowercase()))
}

fn encode_u8(value: u8) -> String {
    format!("{:064x}", value)
}

fn encode_u64(value: u64) -> String {
    format!("{:064x}", value)
}

pub async fn get_wallet_balance(
    query: MonadDataQuery,
    rpc_url: &str,
) -> Result<WalletBalanceResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;

    let client = MonadRpcClient::new(rpc_url);
    let balance_hex = client
        .get_balance(&wallet_address)
        .await
        .map_err(map_client_error)?;

    let balance_wei = hex_to_u128(&balance_hex)?;

    Ok(WalletBalanceResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        balance_mon: wei_to_mon_string(balance_wei),
    })
}

pub async fn get_epoch(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<EpochResponse, MonadDataError> {
    let client = MonadRpcClient::new(rpc_url);

    // getEpoch()
    let data = "0x757991a8";

    let result = client
        .eth_call(staking_contract, data)
        .await
        .map_err(map_client_error)?;

    let epoch = word_to_u64(read_word(&result, 0)?)?;
    let in_epoch_delay_period = word_to_bool(read_word(&result, 1)?)?;

    Ok(EpochResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        epoch,
        in_epoch_delay_period,
    })
}

pub async fn get_staked_balance(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<StakedBalanceResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    let client = MonadRpcClient::new(rpc_url);

    // getDelegator(uint64 validatorId, address delegator)
    //
    // selector:
    // 0x573c1ce0
    //
    // calldata:
    // selector
    // + validatorId encoded as one 32-byte ABI word
    // + delegator address left-padded to one 32-byte ABI word
    let encoded_wallet_address = encode_address(&wallet_address)?;

    let calldata = format!(
        "0x{}{}{}",
        "573c1ce0",
        encode_u64(validator_id),
        encoded_wallet_address,
    );

    let result = client
        .eth_call(staking_contract, &calldata)
        .await
        .map_err(map_client_error)?;

    // getDelegator returns:
    // 0: stake
    // 1: accRewardPerToken
    // 2: unclaimedRewards
    // 3: deltaStake
    // 4: nextDeltaStake
    // 5: deltaEpoch
    // 6: nextDeltaEpoch
    let stake_wei = word_to_u128(read_word(&result, 0)?)?;
    let delta_stake_wei = word_to_u128(read_word(&result, 3)?)?;
    let next_delta_stake_wei = word_to_u128(read_word(&result, 4)?)?;
    let delta_epoch = word_to_u64(read_word(&result, 5)?)?;
    let next_delta_epoch = word_to_u64(read_word(&result, 6)?)?;

    Ok(StakedBalanceResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        staked_balance_mon: wei_to_mon_string(stake_wei),
        delta_stake_mon: Some(wei_to_mon_string(delta_stake_wei)),
        next_delta_stake_mon: Some(wei_to_mon_string(next_delta_stake_wei)),
        delta_epoch: Some(delta_epoch),
        next_delta_epoch: Some(next_delta_epoch),
    })
}

pub async fn get_pending_rewards(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<PendingRewardsResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    let client = MonadRpcClient::new(rpc_url);

    let encoded_wallet_address = encode_address(&wallet_address)?;

    // getDelegator(uint64,address)
    //
    // Returns:
    // 0: stake
    // 1: accRewardPerToken
    // 2: unclaimedRewards
    // 3: deltaStake
    // 4: nextDeltaStake
    // 5: deltaEpoch
    // 6: nextDeltaEpoch
    let calldata = format!(
        "0x{}{}{}",
        "573c1ce0",
        encode_u64(validator_id),
        encoded_wallet_address,
    );

    let result = client
        .eth_call(staking_contract, &calldata)
        .await
        .map_err(map_client_error)?;

    let unclaimed_rewards_wei = word_to_u128(read_word(&result, 2)?)?;

    Ok(PendingRewardsResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        pending_rewards_mon: wei_to_mon_string(unclaimed_rewards_wei),
    })
}

pub async fn get_apy(
    query: MonadDataQuery,
) -> Result<ApyResponse, MonadDataError> {
    Ok(ApyResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        apy: "not_available".to_string(),
        source: "historical_reward_data_required".to_string(),
    })
}

pub async fn get_validator_metadata(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<ValidatorMetadataResponse, MonadDataError> {
    let validator_id = require_validator_id(&query)?;

    let client = MonadRpcClient::new(rpc_url);

    // getValidator(uint64)
    //
    // Returns:
    // 0: authAddress
    // 1: flags
    // 2: stake
    // 3: accRewardPerToken
    // 4: commission
    // 5: unclaimedRewards
    // 6: consensusStake
    // 7: consensusCommission
    // 8: snapshotStake
    // 9: snapshotCommission
    // 10: secpPubkey offset
    // 11: blsPubkey offset
    let calldata = format!(
        "0x{}{}",
        "2b6d639a",
        encode_u64(validator_id),
    );

    let result = client
        .eth_call(staking_contract, &calldata)
        .await
        .map_err(map_client_error)?;

    let auth_address = word_to_address(read_word(&result, 0)?)?;
    let flags = word_to_u64(read_word(&result, 1)?)?;

    let stake_wei = word_to_u128(read_word(&result, 2)?)?;
    let commission = word_to_u128(read_word(&result, 4)?)?;
    let unclaimed_rewards_wei = word_to_u128(read_word(&result, 5)?)?;

    let consensus_stake_wei = word_to_u128(read_word(&result, 6)?)?;
    let consensus_commission = word_to_u128(read_word(&result, 7)?)?;

    let snapshot_stake_wei = word_to_u128(read_word(&result, 8)?)?;
    let snapshot_commission = word_to_u128(read_word(&result, 9)?)?;

    Ok(ValidatorMetadataResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        validator_id,
        auth_address: Some(auth_address),
        flags,
        stake_mon: wei_to_mon_string(stake_wei),

        // Keep commission values raw until Monad's scaling convention
        // is confirmed.
        commission: commission.to_string(),

        consensus_stake_mon: wei_to_mon_string(consensus_stake_wei),
        consensus_commission: consensus_commission.to_string(),

        snapshot_stake_mon: wei_to_mon_string(snapshot_stake_wei),
        snapshot_commission: snapshot_commission.to_string(),

        unclaimed_rewards_mon: wei_to_mon_string(unclaimed_rewards_wei),
    })
}

pub async fn get_withdrawal_request(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<WithdrawalRequestResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;
    let withdraw_id = require_withdraw_id(&query)?;

    let client = MonadRpcClient::new(rpc_url);

    let encoded_wallet_address = encode_address(&wallet_address)?;

    // getWithdrawalRequest(
    //     uint64 validatorId,
    //     address delegator,
    //     uint8 withdrawId
    // )
    //
    // Returns:
    // 0: withdrawalAmount
    // 1: accRewardPerToken
    // 2: withdrawEpoch
    let calldata = format!(
        "0x{}{}{}{}",
        "56fa2045",
        encode_u64(validator_id),
        encoded_wallet_address,
        encode_u8(withdraw_id),
    );

    let result = client
        .eth_call(staking_contract, &calldata)
        .await
        .map_err(map_client_error)?;

    let withdrawal_amount_wei = word_to_u128(read_word(&result, 0)?)?;
    let withdraw_epoch = word_to_u64(read_word(&result, 2)?)?;

    Ok(WithdrawalRequestResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        withdraw_id,
        withdrawal_amount_mon: wei_to_mon_string(withdrawal_amount_wei),
        withdraw_epoch,
    })
}

pub async fn get_account_summary(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<AccountSummaryResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    let network = resolve_network(&query);
    let location = resolve_location(&query);

    let epoch = get_epoch(
        query.clone(),
        rpc_url,
        staking_contract,
    )
    .await?;

    let wallet_balance = get_wallet_balance(
        query.clone(),
        rpc_url,
    )
    .await?;

    let staked_balance = get_staked_balance(
        query.clone(),
        rpc_url,
        staking_contract,
    )
    .await?;

    let pending_rewards = get_pending_rewards(
        query.clone(),
        rpc_url,
        staking_contract,
    )
    .await?;

    let validator = get_validator_metadata(
        query.clone(),
        rpc_url,
        staking_contract,
    )
    .await?;

    let apy = get_apy(query.clone()).await?;

    Ok(AccountSummaryResponse {
        network,
        location,
        wallet_address,
        validator_id,
        epoch,
        wallet_balance_mon: wallet_balance.balance_mon,
        staked_balance_mon: staked_balance.staked_balance_mon,
        pending_rewards_mon: pending_rewards.pending_rewards_mon,
        apy: Some(apy.apy),
        validator: Some(validator),
    })
}