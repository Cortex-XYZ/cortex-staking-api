use actix_web::{get, web, HttpResponse, Responder};
use cortex_auth::{extractor::require_scope, model::Scope};
use cortex_chain_monad::model::{MonadDataQuery, MonadNetwork};
use cortex_services::monad_service;

use crate::{
    errors::error_response,
    extractors::{auth::Authenticated, request_id::get_request_id},
    state::AppState,
};

fn resolve_network(network: Option<MonadNetwork>) -> MonadNetwork {
    network.unwrap_or_default()
}

fn resolve_monad_config<'a>(
    state: &'a AppState,
    network: &MonadNetwork,
) -> Result<(&'a str, &'a str), monad_service::MonadServiceError> {
    let rpc_url = state
        .config
        .monad
        .rpc_url(network)
        .ok_or(monad_service::MonadServiceError::MissingRpcUrl)?;

    let staking_contract = state.config.monad.staking_contract(network);

    Ok((rpc_url, staking_contract))
}

fn monad_service_error_response(
    error: monad_service::MonadServiceError,
    request_id: Option<String>,
) -> HttpResponse {
    match error {
        monad_service::MonadServiceError::MissingRpcUrl => error_response(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "missing_rpc_url",
            "Monad RPC URL is not configured",
            request_id,
        ),

        monad_service::MonadServiceError::MissingStakingContract => error_response(
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            "missing_staking_contract",
            "Monad staking contract is not configured",
            request_id,
        ),

        monad_service::MonadServiceError::MissingWalletAddress => error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing_wallet_address",
            "wallet_address is required",
            request_id,
        ),

        monad_service::MonadServiceError::MissingValidatorId => error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing_validator_id",
            "validator_id is required",
            request_id,
        ),

        monad_service::MonadServiceError::MissingWithdrawId => error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            "missing_withdraw_id",
            "withdraw_id is required",
            request_id,
        ),

        monad_service::MonadServiceError::RpcError => error_response(
            actix_web::http::StatusCode::BAD_GATEWAY,
            "monad_rpc_error",
            "Monad RPC returned an error",
            request_id,
        ),

        monad_service::MonadServiceError::InvalidRpcResponse => error_response(
            actix_web::http::StatusCode::BAD_GATEWAY,
            "invalid_monad_rpc_response",
            "Monad RPC returned an invalid response",
            request_id,
        ),

        monad_service::MonadServiceError::InvalidAmount => error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            "invalid_amount",
            "Invalid MON amount",
            request_id,
        ),

        monad_service::MonadServiceError::InvalidWalletAddress => error_response(
            actix_web::http::StatusCode::BAD_REQUEST,
            "invalid_wallet_address",
            "Invalid wallet address",
            request_id,
        ),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/wallet-balance",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Wallet balance returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/wallet-balance")]
pub async fn wallet_balance(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, _) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_wallet_balance(query, rpc_url).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/epoch",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Epoch returned"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/epoch")]
pub async fn epoch(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_epoch(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/staked-balance",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Staked balance returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/staked-balance")]
pub async fn staked_balance(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_staked_balance(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/pending-rewards",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Pending rewards returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/pending-rewards")]
pub async fn pending_rewards(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_pending_rewards(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/apy",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "APY returned"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope")
    )
)]
#[get("/data/apy")]
pub async fn apy(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    match monad_service::get_apy(query.into_inner()).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/validator-metadata",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Validator metadata returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/validator-metadata")]
pub async fn validator_metadata(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_validator_metadata(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/withdrawal-request",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Withdrawal request returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/withdrawal-request")]
pub async fn withdrawal_request(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_withdrawal_request(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

#[utoipa::path(
    get,
    path = "/monad/data/account-summary",
    tag = "monad",
    params(MonadDataQuery),
    responses(
        (status = 200, description = "Account summary returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 502, description = "Monad RPC error")
    )
)]
#[get("/data/account-summary")]
pub async fn account_summary(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<MonadDataQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);
    require_scope(&auth.0, Scope::Read)?;

    let query = query.into_inner();
    let network = resolve_network(query.network.clone());

    let (rpc_url, staking_contract) = match resolve_monad_config(&state, &network) {
        Ok(config) => config,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    match monad_service::get_account_summary(query, rpc_url, staking_contract).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(error) => Ok(monad_service_error_response(error, request_id)),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(wallet_balance)
        .service(epoch)
        .service(staked_balance)
        .service(pending_rewards)
        .service(apy)
        .service(validator_metadata)
        .service(withdrawal_request)
        .service(account_summary);
}