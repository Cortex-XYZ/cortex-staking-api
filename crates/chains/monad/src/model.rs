use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MonadNetwork {
    Mainnet,
    Testnet,
}

impl Default for MonadNetwork {
    fn default() -> Self {
        Self::Mainnet
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MonadLocation {
    UsDallas,
}

impl Default for MonadLocation {
    fn default() -> Self {
        Self::UsDallas
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MonadTransactionType {
    Stake,
    ClaimRewards,
    CompoundRewards,
    Deactivate,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StakeRequest {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: String,
    pub validator_id: u64,
    pub amount_mon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClaimRewardsRequest {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: String,
    pub validator_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CompoundRewardsRequest {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: String,
    pub validator_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeactivateRequest {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: String,
    pub validator_id: u64,
    pub amount_mon: String,
    pub withdraw_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WithdrawRequest {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: String,
    pub validator_id: u64,
    pub withdraw_id: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedMonadTransactionResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub transaction_type: MonadTransactionType,
    pub unsigned_transaction: UnsignedMonadTransaction,
    pub metadata: MonadTransactionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsignedMonadTransaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: String,
    pub chain_id: u64,
    pub gas_limit: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MonadDataQuery {
    pub network: Option<MonadNetwork>,
    pub location: Option<MonadLocation>,
    pub wallet_address: Option<String>,
    pub validator_id: Option<u64>,
    pub withdraw_id: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonadTransactionMetadata {
    pub wallet_address: String,
    pub validator_id: u64,
    pub amount_mon: Option<String>,
    pub withdraw_id: Option<u8>,
    pub estimated_fee_mon: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EpochResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub epoch: u64,
    pub in_epoch_delay_period: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WalletBalanceResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub wallet_address: String,
    pub balance_mon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StakedBalanceResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub wallet_address: String,
    pub validator_id: u64,
    pub staked_balance_mon: String,
    pub delta_stake_mon: Option<String>,
    pub next_delta_stake_mon: Option<String>,
    pub delta_epoch: Option<u64>,
    pub next_delta_epoch: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PendingRewardsResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub wallet_address: String,
    pub validator_id: u64,
    pub pending_rewards_mon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApyResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub apy: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidatorMetadataResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub validator_id: u64,
    pub auth_address: Option<String>,
    pub flags: u64,
    pub stake_mon: String,
    pub commission: String,
    pub consensus_stake_mon: String,
    pub consensus_commission: String,
    pub snapshot_stake_mon: String,
    pub snapshot_commission: String,
    pub unclaimed_rewards_mon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WithdrawalRequestResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub wallet_address: String,
    pub validator_id: u64,
    pub withdraw_id: u8,
    pub withdrawal_amount_mon: String,
    pub withdraw_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountSummaryResponse {
    pub network: MonadNetwork,
    pub location: MonadLocation,
    pub wallet_address: String,
    pub validator_id: u64,

    pub epoch: EpochResponse,
    pub wallet_balance_mon: String,
    pub staked_balance_mon: String,
    pub pending_rewards_mon: String,
    pub apy: Option<String>,
    pub validator: Option<ValidatorMetadataResponse>,
}