use actix_web::{get, post, delete, patch, web, HttpResponse, Responder};
use cortex_auth::extractor::require_cortex_admin;
use cortex_services::organization_service;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{extractors::auth::Authenticated, state::AppState};

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
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CreateOrganizationRequest>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let organization = organization_service::create_partner_organization(
        &state.db,
        organization_service::CreatePartnerOrganizationInput {
            name: body.name.clone(),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

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
    auth: Authenticated,
    state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let organizations = organization_service::list_organizations(&state.db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<OrganizationResponse> = organizations
        .into_iter()
        .map(OrganizationResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(response))
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
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(organization) =
        organization_service::get_organization_by_id(&state.db, &path.into_inner())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "organization_not_found"
        })));
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
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateOrganizationRequest>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(organization) = organization_service::update_organization(
        &state.db,
        &path.into_inner(),
        organization_service::UpdateOrganizationInput {
            name: body.name.clone(),
            status: body.status.clone(),
        },
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "organization_not_found"
        })));
    };

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
    auth: Authenticated,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let Some(organization) =
        organization_service::delete_organization(&state.db, &path.into_inner())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?
    else {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "organization_not_found"
        })));
    };

    Ok(HttpResponse::Ok().json(OrganizationResponse::from(organization)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_organization)
        .service(list_organizations)
        .service(get_organization_by_id)
        .service(update_organization)
        .service(delete_organization);
}