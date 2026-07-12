use crate::model::{
    ClaimRewardsRequest, CompoundRewardsRequest, DeactivateRequest, MonadLocation, MonadNetwork,
    MonadTransactionMetadata, MonadTransactionType, StakeRequest, UnsignedMonadTransaction,
    UnsignedMonadTransactionResponse, WithdrawRequest,
};

fn chain_id(network: &MonadNetwork) -> u64 {
    match network {
        MonadNetwork::Mainnet => 0,
        MonadNetwork::Testnet => 10143,
    }
}

fn encode_u64(value: u64) -> String {
    format!("{:064x}", value)
}

fn encode_u8(value: u8) -> String {
    format!("{:064x}", value)
}

fn encode_u256_decimal(value: &str) -> String {
    let value = value
        .parse::<u128>()
        .expect("amount must already be converted to wei as decimal string");

    format!("{:064x}", value)
}

fn build_unsigned_transaction(
    network: MonadNetwork,
    location: MonadLocation,
    staking_contract: &str,
    transaction_type: MonadTransactionType,
    wallet_address: String,
    validator_id: u64,
    withdraw_id: Option<u8>,
    amount_mon: Option<String>,
    value: String,
    data: String,
) -> UnsignedMonadTransactionResponse {
    UnsignedMonadTransactionResponse {
        network: network.clone(),
        location,
        transaction_type,
        unsigned_transaction: UnsignedMonadTransaction {
            from: wallet_address.clone(),
            to: staking_contract.to_string(),
            value,
            data,
            chain_id: chain_id(&network),
            gas_limit: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        },
        metadata: MonadTransactionMetadata {
            wallet_address,
            validator_id,
            amount_mon,
            withdraw_id,
            estimated_fee_mon: None,
            notes: vec![
                "Unsigned transaction only. Client wallet must sign and broadcast.".to_string(),
                "Gas fields are placeholders until RPC gas estimation is implemented.".to_string(),
            ],
        },
    }
}

pub fn build_stake_transaction(
    request: StakeRequest,
    staking_contract: &str,
    amount_wei: String,
) -> UnsignedMonadTransactionResponse {
    let network = request.network.unwrap_or_default();
    let location = request.location.unwrap_or_default();

    // delegate(uint64 validatorId), payable
    let data = format!("0x{}{}", "84994fec", encode_u64(request.validator_id));

    build_unsigned_transaction(
        network,
        location,
        staking_contract,
        MonadTransactionType::Stake,
        request.wallet_address,
        request.validator_id,
        None,
        Some(request.amount_mon),
        amount_wei,
        data,
    )
}

pub fn build_claim_rewards_transaction(
    request: ClaimRewardsRequest,
    staking_contract: &str,
) -> UnsignedMonadTransactionResponse {
    let network = request.network.unwrap_or_default();
    let location = request.location.unwrap_or_default();

    // claimRewards(uint64 validatorId)
    let data = format!("0x{}{}", "a76e2ca5", encode_u64(request.validator_id));

    build_unsigned_transaction(
        network,
        location,
        staking_contract,
        MonadTransactionType::ClaimRewards,
        request.wallet_address,
        request.validator_id,
        None,
        None,
        "0".to_string(),
        data,
    )
}

pub fn build_compound_rewards_transaction(
    request: CompoundRewardsRequest,
    staking_contract: &str,
) -> UnsignedMonadTransactionResponse {
    let network = request.network.unwrap_or_default();
    let location = request.location.unwrap_or_default();

    // compound(uint64 validatorId)
    let data = format!("0x{}{}", "b34fea67", encode_u64(request.validator_id));

    build_unsigned_transaction(
        network,
        location,
        staking_contract,
        MonadTransactionType::CompoundRewards,
        request.wallet_address,
        request.validator_id,
        None,
        None,
        "0".to_string(),
        data,
    )
}

pub fn build_deactivate_transaction(
    request: DeactivateRequest,
    staking_contract: &str,
    amount_wei: String,
) -> UnsignedMonadTransactionResponse {
    let network = request.network.unwrap_or_default();
    let location = request.location.unwrap_or_default();

    // undelegate(uint64 validatorId, uint256 amount, uint8 withdrawId)
    let data = format!(
        "0x{}{}{}{}",
        "5cf41514",
        encode_u64(request.validator_id),
        encode_u256_decimal(&amount_wei),
        encode_u8(request.withdraw_id)
    );

    build_unsigned_transaction(
        network,
        location,
        staking_contract,
        MonadTransactionType::Deactivate,
        request.wallet_address,
        request.validator_id,
        Some(request.withdraw_id),
        Some(request.amount_mon),
        "0".to_string(),
        data,
    )
}

pub fn build_withdraw_transaction(
    request: WithdrawRequest,
    staking_contract: &str,
) -> UnsignedMonadTransactionResponse {
    let network = request.network.unwrap_or_default();
    let location = request.location.unwrap_or_default();

    // withdraw(uint64 validatorId, uint8 withdrawId)
    let data = format!(
        "0x{}{}{}",
        "aed2ee73",
        encode_u64(request.validator_id),
        encode_u8(request.withdraw_id)
    );

    build_unsigned_transaction(
        network,
        location,
        staking_contract,
        MonadTransactionType::Withdraw,
        request.wallet_address,
        request.validator_id,
        Some(request.withdraw_id),
        None,
        "0".to_string(),
        data,
    )
}