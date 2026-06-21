use actix_web::{get, post, web, HttpResponse, Responder};
use cortex_auth::extractor::require_cortex_admin;
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

#[utoipa::path(
    post,
    path = "/admin/organizations",
    tag = "admin",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Partner organization created", body = OrganizationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 405, description = "Method Not Allowed"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable"),
        (status = 504, description = "Gateway Timeout")
    )
)]
#[post("/organizations")]
pub async fn create_organization(
    auth: Authenticated,
    state: web::Data<AppState>,
    body: web::Json<CreateOrganizationRequest>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let organization = cortex_db::organization_repository::create_partner_organization(
        &state.db,
        &body.name,
    )
    .await
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(OrganizationResponse {
        id: organization.id,
        name: organization.name,
        kind: organization.kind,
        status: organization.status,
    }))
}

#[utoipa::path(
    get,
    path = "/admin/organizations",
    tag = "admin",
    responses(
        (status = 200, description = "Organizations returned", body = Vec<OrganizationResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 405, description = "Method Not Allowed"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable"),
        (status = 504, description = "Gateway Timeout")
    )
)]
#[get("/organizations")]
pub async fn list_organizations(
    auth: Authenticated,
    state: web::Data<AppState>,
) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    let organizations = cortex_db::organization_repository::list_organizations(&state.db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let response: Vec<OrganizationResponse> = organizations
        .into_iter()
        .map(|organization| OrganizationResponse {
            id: organization.id,
            name: organization.name,
            kind: organization.kind,
            status: organization.status,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_organization)
        .service(list_organizations);
}