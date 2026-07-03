use actix_web::{delete, get, patch, web, HttpResponse, Responder};
use cortex_auth::extractor::require_cortex_admin;
use cortex_services::{
    audit_actions, 
    audit_service,
    user_service,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{extractors::auth::Authenticated, state::AppState};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub social_provider: Option<String>,
    pub social_provider_user_id: Option<String>,
    pub status: String,
    pub key_limit: i32,
    pub rate_limit_tier: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub status: Option<String>,
    pub key_limit: Option<i32>,
    pub rate_limit_tier: Option<String>,
}

impl From<cortex_db::user_repository::UserRecord> for UserResponse {
    fn from(user: cortex_db::user_repository::UserRecord) -> Self {
        Self {
            id: user.id,
            email: user.email,
            wallet_address: user.wallet_address,
            social_provider: user.social_provider,
            social_provider_user_id: user.social_provider_user_id,
            status: user.status,
            key_limit: user.key_limit,
            rate_limit_tier: user.rate_limit_tier,
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/users",
    tag = "admin",
    responses(
        (status = 200, description = "Users returned", body = Vec<UserResponse>),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required")
    )
)]
#[get("/users")]
pub async fn list_users(
    auth: Authenticated,
    state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let users = user_service::list_users(&state.db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/admin/users/{id}",
    tag = "admin",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User returned", body = UserResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "User not found")
    )
)]
#[get("/users/{id}")]
pub async fn get_user_by_id(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(user) = user_service::get_user_by_id(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found"
        })));
    };

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

#[utoipa::path(
    patch,
    path = "/admin/users/{id}",
    tag = "admin",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = UserResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "User not found")
    )
)]
#[patch("/users/{id}")]
pub async fn update_user(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateUserRequest>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(user) = user_service::update_user(
        &state.db,
        &path.into_inner(),
        user_service::UpdateUserServiceInput {
            email: body.email.clone(),
            wallet_address: body.wallet_address.clone(),
            status: body.status.clone(),
            key_limit: body.key_limit,
            rate_limit_tier: body.rate_limit_tier.clone(),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found"
        })));
    };

    audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::USER_UPDATED.to_string(),
            resource_type: "user".to_string(),
            resource_id: Some(user.id.clone()),
            ip_address: None,
            request_id: None,
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "wallet_address": user.wallet_address,
                "social_provider": user.social_provider,
                "social_provider_user_id": user.social_provider_user_id,
                "status": user.status,
                "key_limit": user.key_limit,
                "rate_limit_tier": user.rate_limit_tier,
            })),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

#[utoipa::path(
    delete,
    path = "/admin/users/{id}",
    tag = "admin",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted", body = UserResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "User not found")
    )
)]
#[delete("/users/{id}")]
pub async fn delete_user(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(user) = user_service::delete_user(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "user_not_found"
        })));
    };

    audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::USER_DELETED.to_string(),
            resource_type: "user".to_string(),
            resource_id: Some(user.id.clone()),
            ip_address: None,
            request_id: None,
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": user.id,
                "email": user.email,
                "wallet_address": user.wallet_address,
                "status": user.status,
                "key_limit": user.key_limit,
                "rate_limit_tier": user.rate_limit_tier,
            })),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(UserResponse::from(user)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(list_users)
        .service(get_user_by_id)
        .service(update_user)
        .service(delete_user);
}