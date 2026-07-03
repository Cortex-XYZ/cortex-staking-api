use cortex_db::organization_repository::{self, OrganizationRecord};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct CreatePartnerOrganizationInput {
    pub name: String,
}

pub async fn create_partner_organization(
    db: &PgPool,
    input: CreatePartnerOrganizationInput,
) -> Result<OrganizationRecord, sqlx::Error> {
    organization_repository::create_partner_organization(db, &input.name).await
}

pub async fn list_organizations(
    db: &PgPool,
) -> Result<Vec<OrganizationRecord>, sqlx::Error> {
    organization_repository::list_organizations(db).await
}

pub async fn get_organization_by_id(
    db: &PgPool,
    organization_id: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    organization_repository::get_organization_by_id(db, organization_id).await
}

#[derive(Debug, Clone)]
pub struct UpdateOrganizationInput {
    pub name: Option<String>,
    pub status: Option<String>,
}

pub async fn update_organization(
    db: &PgPool,
    organization_id: &str,
    input: UpdateOrganizationInput,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    organization_repository::update_organization(
        db,
        organization_id,
        input.name.as_deref(),
        input.status.as_deref(),
    )
    .await
}

pub async fn delete_organization(
    db: &PgPool,
    organization_id: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    organization_repository::soft_delete_organization(db, organization_id).await
}