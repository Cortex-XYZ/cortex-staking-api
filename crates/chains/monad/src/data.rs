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
    match error {
        MonadClientError::InvalidResponse => MonadDataError::InvalidRpcResponse,
        MonadClientError::RpcRequestFailed | MonadClientError::RpcError(_) => {
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

fn word_to_bool(word: &str) -> Result<bool, MonadDataError> {
    Ok(word_to_u64(word)? != 0)
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
    let data = "0x2e73b04d";

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
    _rpc_url: &str,
    _staking_contract: &str,
) -> Result<StakedBalanceResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    Ok(StakedBalanceResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        staked_balance_mon: "0".to_string(),
        delta_stake_mon: Some("0".to_string()),
        next_delta_stake_mon: Some("0".to_string()),
        delta_epoch: Some(0),
        next_delta_epoch: Some(0),
    })
}

pub async fn get_pending_rewards(
    query: MonadDataQuery,
    _rpc_url: &str,
    _staking_contract: &str,
) -> Result<PendingRewardsResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    Ok(PendingRewardsResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        pending_rewards_mon: "0".to_string(),
    })
}

pub async fn get_apy(query: MonadDataQuery) -> Result<ApyResponse, MonadDataError> {
    Ok(ApyResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        apy: "0".to_string(),
        source: "not_available".to_string(),
    })
}

pub async fn get_validator_metadata(
    query: MonadDataQuery,
    _rpc_url: &str,
    _staking_contract: &str,
) -> Result<ValidatorMetadataResponse, MonadDataError> {
    let validator_id = require_validator_id(&query)?;

    Ok(ValidatorMetadataResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        validator_id,
        auth_address: None,
        flags: 0,
        stake_mon: "0".to_string(),
        commission: "0".to_string(),
        consensus_stake_mon: "0".to_string(),
        consensus_commission: "0".to_string(),
        snapshot_stake_mon: "0".to_string(),
        snapshot_commission: "0".to_string(),
        unclaimed_rewards_mon: "0".to_string(),
    })
}

pub async fn get_withdrawal_request(
    query: MonadDataQuery,
    _rpc_url: &str,
    _staking_contract: &str,
) -> Result<WithdrawalRequestResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;
    let withdraw_id = require_withdraw_id(&query)?;

    Ok(WithdrawalRequestResponse {
        network: resolve_network(&query),
        location: resolve_location(&query),
        wallet_address,
        validator_id,
        withdraw_id,
        withdrawal_amount_mon: "0".to_string(),
        withdraw_epoch: 0,
    })
}

pub async fn get_account_summary(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<AccountSummaryResponse, MonadDataError> {
    let wallet_address = require_wallet_address(&query)?;
    let validator_id = require_validator_id(&query)?;

    let epoch = get_epoch(query.clone(), rpc_url, staking_contract).await?;
    let wallet_balance = get_wallet_balance(query.clone(), rpc_url).await?;

    let network = resolve_network(&query);
    let location = resolve_location(&query);

    Ok(AccountSummaryResponse {
        network: network.clone(),
        location: location.clone(),
        wallet_address,
        validator_id,
        epoch,
        wallet_balance_mon: wallet_balance.balance_mon,
        staked_balance_mon: "0".to_string(),
        pending_rewards_mon: "0".to_string(),
        apy: Some("0".to_string()),
        validator: None,
    })
}