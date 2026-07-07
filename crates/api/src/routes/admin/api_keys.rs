use actix_web::{delete, get, post, web, HttpResponse, Responder};
use cortex_auth::extractor::require_cortex_admin;
use cortex_db::pagination::DbPagination;
use cortex_services::{api_key_service, audit_actions, audit_service};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    errors::{internal_error, not_found},
    extractors::{auth::Authenticated, request_id::get_request_id},
    pagination::{
        PaginatedResponse, 
        PaginationMeta, 
        PaginationQuery,
        SortDirection,
    },
    state::AppState,
};

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

#[derive(Debug, Deserialize)]
pub struct ListApiKeysQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub direction: Option<String>,

    pub organization_id: Option<String>,
    pub status: Option<String>,
    pub scope: Option<String>,
    pub created_after: Option<String>,
    pub last_used_after: Option<String>,
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

fn to_db_pagination(query: PaginationQuery) -> (DbPagination, i64, i64) {
    let pagination = query.into_pagination();

    let sort_direction = match pagination.direction {
        SortDirection::Asc => "asc",
        SortDirection::Desc => "desc",
    };

    (
        DbPagination::new(
            pagination.page_size,
            pagination.offset,
            pagination.sort.unwrap_or_else(|| "created_at".to_string()),
            sort_direction,
        ),
        pagination.page,
        pagination.page_size,
    )
}

fn total_pages(total_items: i64, page_size: i64) -> i64 {
    if total_items == 0 {
        0
    } else {
        (total_items + page_size - 1) / page_size
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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CreateApiKeyRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let created = match api_key_service::create_organization_api_key(
        &state.db,
        api_key_service::CreateOrganizationApiKeyServiceInput {
            organization_id: body.organization_id.clone(),
            name: body.name.clone(),
            scopes: body.scopes.clone(),
            rate_limit_per_minute: body.rate_limit_per_minute.unwrap_or(120),
        },
    )
    .await
    {
        Ok(created) => created,
        Err(_) => return Ok(internal_error(request_id)),
    };

    if audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::API_KEY_CREATED.to_string(),
            resource_type: "api_key".to_string(),
            resource_id: Some(created.api_key.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": created.api_key.id.clone(),
                "owner_type": created.api_key.owner_type.clone(),
                "organization_id": created.api_key.organization_id.clone(),
                "user_id": created.api_key.user_id.clone(),
                "name": created.api_key.name.clone(),
                "key_prefix": created.api_key.key_prefix.clone(),
                "status": created.api_key.status.clone(),
                "rate_limit_per_minute": created.api_key.rate_limit_per_minute,
                "scopes": body.scopes.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<ListApiKeysQuery>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let query = query.into_inner();

    let (db_pagination, page, page_size) = to_db_pagination(PaginationQuery {
        page: query.page,
        page_size: query.page_size,
        sort: query.sort,
        direction: query.direction,
    });

    let paginated = match api_key_service::list_api_keys(
        &state.db,
        api_key_service::ListApiKeysInput {
            pagination: db_pagination,
            filters: cortex_db::api_key_repository::ApiKeyFilters {
                organization_id: query.organization_id,
                status: query.status,
                scope: query.scope,
                created_after: query.created_after,
                last_used_after: query.last_used_after,
            },
        },
    )
    .await
    {
        Ok(paginated) => paginated,
        Err(_) => return Ok(internal_error(request_id)),
    };

    let response: Vec<ApiKeyResponse> = paginated
        .items
        .into_iter()
        .map(ApiKeyResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(PaginatedResponse {
        data: response,
        pagination: PaginationMeta {
            page,
            page_size,
            total_items: paginated.total_items,
            total_pages: total_pages(paginated.total_items, page_size),
        },
    }))
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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let key = match api_key_service::revoke_api_key(&state.db, &path.into_inner()).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return Ok(not_found(
                "api_key_not_found",
                "API key not found",
                request_id,
            ));
        }
        Err(_) => return Ok(internal_error(request_id)),
    };

    if audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::API_KEY_REVOKED.to_string(),
            resource_type: "api_key".to_string(),
            resource_id: Some(key.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": key.id.clone(),
                "key_prefix": key.key_prefix.clone(),
                "status": key.status.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let key = match api_key_service::get_api_key_by_id(&state.db, &path.into_inner()).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return Ok(not_found(
                "api_key_not_found",
                "API key not found",
                request_id,
            ));
        }
        Err(_) => return Ok(internal_error(request_id)),
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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let rotated = match api_key_service::rotate_api_key(&state.db, &path.into_inner()).await {
        Ok(Some(rotated)) => rotated,
        Ok(None) => {
            return Ok(not_found(
                "api_key_not_found",
                "API key not found",
                request_id,
            ));
        }
        Err(_) => return Ok(internal_error(request_id)),
    };

    if audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::API_KEY_ROTATED.to_string(),
            resource_type: "api_key".to_string(),
            resource_id: Some(rotated.api_key.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": rotated.api_key.id.clone(),
                "key_prefix": rotated.api_key.key_prefix.clone(),
                "status": rotated.api_key.status.clone(),
                "name": rotated.api_key.name.clone(),
                "rate_limit_per_minute": rotated.api_key.rate_limit_per_minute,
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

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
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let key = match api_key_service::delete_api_key(&state.db, &path.into_inner()).await {
        Ok(Some(key)) => key,
        Ok(None) => {
            return Ok(not_found(
                "api_key_not_found",
                "API key not found",
                request_id,
            ));
        }
        Err(_) => return Ok(internal_error(request_id)),
    };

    if audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::API_KEY_DELETED.to_string(),
            resource_type: "api_key".to_string(),
            resource_id: Some(key.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": key.id.clone(),
                "key_prefix": key.key_prefix.clone(),
                "status": key.status.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

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