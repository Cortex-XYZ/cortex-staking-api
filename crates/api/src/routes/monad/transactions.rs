use actix_web::{post, web, HttpResponse, Responder};
use cortex_auth::{extractor::require_scope, model::Scope};
use cortex_chain_monad::model::{
    ClaimRewardsRequest, CompoundRewardsRequest, DeactivateRequest, MonadNetwork, StakeRequest,
    WithdrawRequest,
};
use cortex_services::monad_service;

use crate::{
    errors::error_response,
    extractors::{auth::Authenticated, request_id::get_request_id},
    state::AppState,
};

fn resolve_network(network: Option<MonadNetwork>) -> MonadNetwork {
    network.unwrap_or_default()
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
    }
}

#[utoipa::path(
    post,
    path = "/monad/transactions/stake",
    tag = "monad",
    request_body = StakeRequest,
    responses(
        (status = 200, description = "Unsigned stake transaction returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/transactions/stake")]
pub async fn stake(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<StakeRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_scope(&auth.0, Scope::Write)?;

    let request = body.into_inner();
    let network = resolve_network(request.network.clone());
    let staking_contract = state.config.monad.staking_contract(&network);

    let response = match monad_service::build_stake_transaction(request, staking_contract) {
        Ok(response) => response,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/monad/transactions/claim-rewards",
    tag = "monad",
    request_body = ClaimRewardsRequest,
    responses(
        (status = 200, description = "Unsigned claim rewards transaction returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/transactions/claim-rewards")]
pub async fn claim_rewards(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<ClaimRewardsRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_scope(&auth.0, Scope::Write)?;

    let request = body.into_inner();
    let network = resolve_network(request.network.clone());
    let staking_contract = state.config.monad.staking_contract(&network);

    let response = match monad_service::build_claim_rewards_transaction(request, staking_contract) {
        Ok(response) => response,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/monad/transactions/compound-rewards",
    tag = "monad",
    request_body = CompoundRewardsRequest,
    responses(
        (status = 200, description = "Unsigned compound rewards transaction returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/transactions/compound-rewards")]
pub async fn compound_rewards(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CompoundRewardsRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_scope(&auth.0, Scope::Write)?;

    let request = body.into_inner();
    let network = resolve_network(request.network.clone());
    let staking_contract = state.config.monad.staking_contract(&network);

    let response =
        match monad_service::build_compound_rewards_transaction(request, staking_contract) {
            Ok(response) => response,
            Err(error) => return Ok(monad_service_error_response(error, request_id)),
        };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/monad/transactions/deactivate",
    tag = "monad",
    request_body = DeactivateRequest,
    responses(
        (status = 200, description = "Unsigned deactivate transaction returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/transactions/deactivate")]
pub async fn deactivate(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<DeactivateRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_scope(&auth.0, Scope::Write)?;

    let request = body.into_inner();
    let network = resolve_network(request.network.clone());
    let staking_contract = state.config.monad.staking_contract(&network);

    let response = match monad_service::build_deactivate_transaction(request, staking_contract) {
        Ok(response) => response,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/monad/transactions/withdraw",
    tag = "monad",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Unsigned withdraw transaction returned"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Missing required scope"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/transactions/withdraw")]
pub async fn withdraw(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<WithdrawRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_scope(&auth.0, Scope::Write)?;

    let request = body.into_inner();
    let network = resolve_network(request.network.clone());
    let staking_contract = state.config.monad.staking_contract(&network);

    let response = match monad_service::build_withdraw_transaction(request, staking_contract) {
        Ok(response) => response,
        Err(error) => return Ok(monad_service_error_response(error, request_id)),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(stake)
        .service(claim_rewards)
        .service(compound_rewards)
        .service(deactivate)
        .service(withdraw);
}