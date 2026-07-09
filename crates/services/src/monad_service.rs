use cortex_chain_monad::{
    data,
    model::{
        AccountSummaryResponse, ApyResponse, ClaimRewardsRequest, CompoundRewardsRequest,
        DeactivateRequest, EpochResponse, MonadDataQuery, PendingRewardsResponse, StakeRequest,
        StakedBalanceResponse, UnsignedMonadTransactionResponse, ValidatorMetadataResponse,
        WalletBalanceResponse, WithdrawRequest, WithdrawalRequestResponse,
    },
    transactions,
};

use crate::monad_service::MonadServiceError::{
    InvalidAmount, MissingRpcUrl, MissingStakingContract,
};

#[derive(Debug)]
pub enum MonadServiceError {
    InvalidAmount,
    MissingRpcUrl,
    MissingStakingContract,
    MissingWalletAddress,
    MissingValidatorId,
    MissingWithdrawId,
    RpcError,
    InvalidRpcResponse,
}

impl From<data::MonadDataError> for MonadServiceError {
    fn from(error: data::MonadDataError) -> Self {
        match error {
            data::MonadDataError::MissingWalletAddress => Self::MissingWalletAddress,
            data::MonadDataError::MissingValidatorId => Self::MissingValidatorId,
            data::MonadDataError::MissingWithdrawId => Self::MissingWithdrawId,
            data::MonadDataError::RpcError => Self::RpcError,
            data::MonadDataError::InvalidRpcResponse => Self::InvalidRpcResponse,
        }
    }
}

fn mon_to_wei(amount_mon: &str) -> Result<String, MonadServiceError> {
    let parts: Vec<&str> = amount_mon.split('.').collect();

    if parts.len() > 2 {
        return Err(InvalidAmount);
    }

    let whole = parts[0].parse::<u128>().map_err(|_| InvalidAmount)?;

    let fractional = if parts.len() == 2 {
        let frac = parts[1];

        if frac.len() > 18 || !frac.chars().all(|c| c.is_ascii_digit()) {
            return Err(InvalidAmount);
        }

        let padded = format!("{:0<18}", frac);
        padded.parse::<u128>().map_err(|_| InvalidAmount)?
    } else {
        0
    };

    let wei = whole
        .checked_mul(1_000_000_000_000_000_000u128)
        .and_then(|value| value.checked_add(fractional))
        .ok_or(InvalidAmount)?;

    Ok(wei.to_string())
}

fn require_staking_contract(staking_contract: &str) -> Result<(), MonadServiceError> {
    if staking_contract.is_empty() {
        Err(MissingStakingContract)
    } else {
        Ok(())
    }
}

fn require_rpc_url(rpc_url: &str) -> Result<(), MonadServiceError> {
    if rpc_url.is_empty() {
        Err(MissingRpcUrl)
    } else {
        Ok(())
    }
}

pub fn build_stake_transaction(
    request: StakeRequest,
    staking_contract: &str,
) -> Result<UnsignedMonadTransactionResponse, MonadServiceError> {
    require_staking_contract(staking_contract)?;

    let amount_wei = mon_to_wei(&request.amount_mon)?;

    Ok(transactions::build_stake_transaction(
        request,
        staking_contract,
        amount_wei,
    ))
}

pub fn build_claim_rewards_transaction(
    request: ClaimRewardsRequest,
    staking_contract: &str,
) -> Result<UnsignedMonadTransactionResponse, MonadServiceError> {
    require_staking_contract(staking_contract)?;

    Ok(transactions::build_claim_rewards_transaction(
        request,
        staking_contract,
    ))
}

pub fn build_compound_rewards_transaction(
    request: CompoundRewardsRequest,
    staking_contract: &str,
) -> Result<UnsignedMonadTransactionResponse, MonadServiceError> {
    require_staking_contract(staking_contract)?;

    Ok(transactions::build_compound_rewards_transaction(
        request,
        staking_contract,
    ))
}

pub fn build_deactivate_transaction(
    request: DeactivateRequest,
    staking_contract: &str,
) -> Result<UnsignedMonadTransactionResponse, MonadServiceError> {
    require_staking_contract(staking_contract)?;

    let amount_wei = mon_to_wei(&request.amount_mon)?;

    Ok(transactions::build_deactivate_transaction(
        request,
        staking_contract,
        amount_wei,
    ))
}

pub fn build_withdraw_transaction(
    request: WithdrawRequest,
    staking_contract: &str,
) -> Result<UnsignedMonadTransactionResponse, MonadServiceError> {
    require_staking_contract(staking_contract)?;

    Ok(transactions::build_withdraw_transaction(
        request,
        staking_contract,
    ))
}

pub async fn get_wallet_balance(
    query: MonadDataQuery,
    rpc_url: &str,
) -> Result<WalletBalanceResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;

    Ok(data::get_wallet_balance(query, rpc_url).await?)
}

pub async fn get_epoch(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<EpochResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_epoch(query, rpc_url, staking_contract).await?)
}

pub async fn get_staked_balance(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<StakedBalanceResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_staked_balance(query, rpc_url, staking_contract).await?)
}

pub async fn get_pending_rewards(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<PendingRewardsResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_pending_rewards(query, rpc_url, staking_contract).await?)
}

pub async fn get_apy(query: MonadDataQuery) -> Result<ApyResponse, MonadServiceError> {
    Ok(data::get_apy(query).await?)
}

pub async fn get_validator_metadata(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<ValidatorMetadataResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_validator_metadata(query, rpc_url, staking_contract).await?)
}

pub async fn get_withdrawal_request(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<WithdrawalRequestResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_withdrawal_request(query, rpc_url, staking_contract).await?)
}

pub async fn get_account_summary(
    query: MonadDataQuery,
    rpc_url: &str,
    staking_contract: &str,
) -> Result<AccountSummaryResponse, MonadServiceError> {
    require_rpc_url(rpc_url)?;
    require_staking_contract(staking_contract)?;

    Ok(data::get_account_summary(query, rpc_url, staking_contract).await?)
}