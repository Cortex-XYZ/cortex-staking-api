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