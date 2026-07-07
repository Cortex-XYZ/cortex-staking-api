use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use cortex_auth::extractor::require_cortex_admin;
use cortex_db::pagination::DbPagination;
use cortex_services::{audit_actions, audit_service, organization_service};
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
pub struct CreateOrganizationRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrganizationResponse {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListOrganizationsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub direction: Option<String>,

    pub status: Option<String>,
    pub kind: Option<String>,
    pub name: Option<String>,
}

impl From<cortex_db::organization_repository::OrganizationRecord> for OrganizationResponse {
    fn from(organization: cortex_db::organization_repository::OrganizationRecord) -> Self {
        Self {
            id: organization.id,
            name: organization.name,
            kind: organization.kind,
            status: organization.status,
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
    path = "/admin/organizations",
    tag = "admin",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Partner organization created", body = OrganizationResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[post("/organizations")]
pub async fn create_organization(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CreateOrganizationRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let organization = match organization_service::create_partner_organization(
        &state.db,
        organization_service::CreatePartnerOrganizationInput {
            name: body.name.clone(),
        },
    )
    .await
    {
        Ok(organization) => organization,
        Err(_) => return Ok(internal_error(request_id)),
    };

    if audit_service::record_admin_action(
        &state.db,
        audit_service::RecordAuditLogInput {
            actor_api_key_id: Some(auth.0.api_key_id.clone()),
            actor_organization_id: auth.0.organization_id.clone(),
            action: audit_actions::ORGANIZATION_CREATED.to_string(),
            resource_type: "organization".to_string(),
            resource_id: Some(organization.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": organization.id.clone(),
                "name": organization.name.clone(),
                "kind": organization.kind.clone(),
                "status": organization.status.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

    Ok(HttpResponse::Created().json(OrganizationResponse::from(organization)))
}

#[utoipa::path(
    get,
    path = "/admin/organizations",
    tag = "admin",
    responses(
        (status = 200, description = "Organizations returned", body = Vec<OrganizationResponse>),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[get("/organizations")]
pub async fn list_organizations(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    query: web::Query<ListOrganizationsQuery>,
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

    let paginated = match organization_service::list_organizations(
        &state.db,
        organization_service::ListOrganizationsInput {
            pagination: db_pagination,
            filters: cortex_db::organization_repository::OrganizationFilters {
                status: query.status,
                kind: query.kind,
                name: query.name,
            },
        },
    )
    .await
    {
        Ok(paginated) => paginated,
        Err(_) => return Ok(internal_error(request_id)),
    };

    let response: Vec<OrganizationResponse> = paginated
        .items
        .into_iter()
        .map(OrganizationResponse::from)
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
    get,
    path = "/admin/organizations/{id}",
    tag = "admin",
    params(("id" = String, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Organization returned", body = OrganizationResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "Organization not found")
    )
)]
#[get("/organizations/{id}")]
pub async fn get_organization_by_id(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let organization =
        match organization_service::get_organization_by_id(&state.db, &path.into_inner()).await {
            Ok(Some(organization)) => organization,
            Ok(None) => {
                return Ok(not_found(
                    "organization_not_found",
                    "Organization not found",
                    request_id,
                ));
            }
            Err(_) => return Ok(internal_error(request_id)),
        };

    Ok(HttpResponse::Ok().json(OrganizationResponse::from(organization)))
}

#[utoipa::path(
    patch,
    path = "/admin/organizations/{id}",
    tag = "admin",
    params(("id" = String, Path, description = "Organization ID")),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated", body = OrganizationResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "Organization not found")
    )
)]
#[patch("/organizations/{id}")]
pub async fn update_organization(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateOrganizationRequest>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let organization = match organization_service::update_organization(
        &state.db,
        &path.into_inner(),
        organization_service::UpdateOrganizationInput {
            name: body.name.clone(),
            status: body.status.clone(),
        },
    )
    .await
    {
        Ok(Some(organization)) => organization,
        Ok(None) => {
            return Ok(not_found(
                "organization_not_found",
                "Organization not found",
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
            action: audit_actions::ORGANIZATION_UPDATED.to_string(),
            resource_type: "organization".to_string(),
            resource_id: Some(organization.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": organization.id.clone(),
                "name": organization.name.clone(),
                "kind": organization.kind.clone(),
                "status": organization.status.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

    Ok(HttpResponse::Ok().json(OrganizationResponse::from(organization)))
}

#[utoipa::path(
    delete,
    path = "/admin/organizations/{id}",
    tag = "admin",
    params(("id" = String, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "Organization deleted", body = OrganizationResponse),
        (status = 401, description = "Missing or invalid API key"),
        (status = 403, description = "Cortex admin key required"),
        (status = 404, description = "Organization not found")
    )
)]
#[delete("/organizations/{id}")]
pub async fn delete_organization(
    req: actix_web::HttpRequest,
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let request_id = get_request_id(&req);

    require_cortex_admin(&auth.0)?;

    let organization =
        match organization_service::delete_organization(&state.db, &path.into_inner()).await {
            Ok(Some(organization)) => organization,
            Ok(None) => {
                return Ok(not_found(
                    "organization_not_found",
                    "Organization not found",
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
            action: audit_actions::ORGANIZATION_DELETED.to_string(),
            resource_type: "organization".to_string(),
            resource_id: Some(organization.id.clone()),
            ip_address: None,
            request_id: request_id.clone(),
            old_values: None,
            new_values: Some(serde_json::json!({
                "id": organization.id.clone(),
                "name": organization.name.clone(),
                "kind": organization.kind.clone(),
                "status": organization.status.clone(),
            })),
        },
    )
    .await
    .is_err()
    {
        return Ok(internal_error(request_id));
    }

    Ok(HttpResponse::Ok().json(OrganizationResponse::from(organization)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_organization)
        .service(list_organizations)
        .service(get_organization_by_id)
        .service(update_organization)
        .service(delete_organization);
}