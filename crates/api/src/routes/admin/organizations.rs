use actix_web::{get, post, web, HttpResponse, Responder};
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

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_organization)
        .service(list_organizations);
}