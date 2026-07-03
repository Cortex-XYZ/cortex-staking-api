use actix_web::{HttpResponse, Responder, get, post, delete, web};
use cortex_auth::extractor::require_cortex_admin;
use cortex_services::api_key_service;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{extractors::auth::Authenticated, state::AppState};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub organization_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyResponse {
    pub id: String,
    pub owner_type: String,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub name: String,
    pub key_prefix: String,
    pub status: String,
    pub rate_limit_per_minute: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKeyResponse,
    pub token: String,
}

impl From<cortex_db::api_key_repository::ApiKeyRecord> for ApiKeyResponse {
    fn from(key: cortex_db::api_key_repository::ApiKeyRecord) -> Self {
        Self {
            id: key.id,
            owner_type: key.owner_type,
            organization_id: key.organization_id,
            user_id: key.user_id,
            name: key.name,
            key_prefix: key.key_prefix,
            status: key.status,
            rate_limit_per_minute: key.rate_limit_per_minute,
        }
    }
}

#[utoipa::path(
    post,
    path = "/admin/api-keys",
    tag = "admin",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = CreateApiKeyResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required")
    )
)]
#[post("/api-keys")]
pub async fn create_api_key(
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CreateApiKeyRequest>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let created = api_key_service::create_organization_api_key(
        &state.db,
        api_key_service::CreateOrganizationApiKeyServiceInput {
            organization_id: body.organization_id.clone(),
            name: body.name.clone(),
            scopes: body.scopes.clone(),
            rate_limit_per_minute: body.rate_limit_per_minute.unwrap_or(120),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(CreateApiKeyResponse {
        api_key: created.api_key.into(),
        token: created.token,
    }))
}

#[utoipa::path(
    get,
    path = "/admin/api-keys",
    tag = "admin",
    responses(
        (status = 200, description = "API keys returned", body = Vec<ApiKeyResponse>),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required")
    )
)]
#[get("/api-keys")]
pub async fn list_api_keys(
    auth: Authenticated,
    state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let keys = api_key_service::list_api_keys(&state.db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<ApiKeyResponse> = keys.into_iter().map(ApiKeyResponse::from).collect();

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    post,
    path = "/admin/api-keys/{id}/revoke",
    tag = "admin",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key revoked", body = ApiKeyResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "API key not found")
    )
)]
#[post("/api-keys/{id}/revoke")]
pub async fn revoke_api_key(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(key) = api_key_service::revoke_api_key(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "api_key_not_found"
        })));
    };

    Ok(HttpResponse::Ok().json(ApiKeyResponse::from(key)))
}

#[utoipa::path(
    get,
    path = "/admin/api-keys/{id}",
    tag = "admin",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key returned", body = ApiKeyResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "API key not found")
    )
)]
#[get("/api-keys/{id}")]
pub async fn get_api_key_by_id(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(key) = api_key_service::get_api_key_by_id(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "api_key_not_found"
        })));
    };

    Ok(HttpResponse::Ok().json(ApiKeyResponse::from(key)))
}

#[utoipa::path(
    post,
    path = "/admin/api-keys/{id}/rotate",
    tag = "admin",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 201, description = "API key rotated", body = CreateApiKeyResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "API key not found")
    )
)]
#[post("/api-keys/{id}/rotate")]
pub async fn rotate_api_key(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(rotated) = api_key_service::rotate_api_key(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "api_key_not_found"
        })));
    };

    Ok(HttpResponse::Created().json(CreateApiKeyResponse {
        api_key: rotated.api_key.into(),
        token: rotated.token,
    }))
}

#[utoipa::path(
    delete,
    path = "/admin/api-keys/{id}",
    tag = "admin",
    params(
        ("id" = String, Path, description = "API key ID")
    ),
    responses(
        (status = 200, description = "API key deleted", body = ApiKeyResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "API key not found")
    )
)]
#[delete("/api-keys/{id}")]
pub async fn delete_api_key(
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(key) = api_key_service::delete_api_key(&state.db, &path.into_inner())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "api_key_not_found"
        })));
    };

    Ok(HttpResponse::Ok().json(ApiKeyResponse::from(key)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_api_key)
        .service(list_api_keys)
        .service(get_api_key_by_id)
        .service(rotate_api_key)
        .service(revoke_api_key)
        .service(delete_api_key);
}
