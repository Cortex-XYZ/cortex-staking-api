use crate::pagination::DbPagination;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct OrganizationRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct PaginatedOrganizations {
    pub items: Vec<OrganizationRecord>,
    pub total_items: i64,
}

pub async fn create_partner_organization(
    db: &PgPool,
    name: &str,
) -> Result<OrganizationRecord, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO organizations (name, kind)
        VALUES ($1, 'partner')
        RETURNING id::text, name, kind, status
        "#,
    )
    .bind(name)
    .fetch_one(db)
    .await?;

    row_to_organization_record(row)
}

pub async fn list_organizations(
    db: &PgPool,
    pagination: DbPagination,
) -> Result<PaginatedOrganizations, sqlx::Error> {
    let total_items: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM organizations
        "#,
    )
    .fetch_one(db)
    .await?;

    let sort_column = match pagination.sort_column.as_str() {
        "name" => "name",
        "kind" => "kind",
        "status" => "status",
        "created_at" => "created_at",
        _ => "created_at",
    };

    let sort_direction = match pagination.sort_direction.as_str() {
        "asc" => "ASC",
        _ => "DESC",
    };

    let query = format!(
        r#"
        SELECT id::text, name, kind, status
        FROM organizations
        ORDER BY {} {}
        LIMIT $1 OFFSET $2
        "#,
        sort_column, sort_direction
    );

    let rows = sqlx::query(&query)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(db)
        .await?;

    let items = rows
        .into_iter()
        .map(row_to_organization_record)
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(PaginatedOrganizations { items, total_items })
}

pub async fn get_organization_by_id(
    db: &PgPool,
    organization_id: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id::text, name, kind, status
        FROM organizations
        WHERE id = $1::uuid
        "#,
    )
    .bind(organization_id)
    .fetch_optional(db)
    .await?;

    row.map(row_to_organization_record).transpose()
}

pub async fn update_organization(
    db: &PgPool,
    organization_id: &str,
    name: Option<&str>,
    status: Option<&str>,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE organizations
        SET
            name = COALESCE($2, name),
            status = COALESCE($3, status),
            updated_at = now()
        WHERE id = $1::uuid
        RETURNING id::text, name, kind, status
        "#,
    )
    .bind(organization_id)
    .bind(name)
    .bind(status)
    .fetch_optional(db)
    .await?;

    row.map(row_to_organization_record).transpose()
}

pub async fn soft_delete_organization(
    db: &PgPool,
    organization_id: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    update_organization(db, organization_id, None, Some("deleted")).await
}

fn row_to_organization_record(
    row: sqlx::postgres::PgRow,
) -> Result<OrganizationRecord, sqlx::Error> {
    Ok(OrganizationRecord {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        kind: row.try_get("kind")?,
        status: row.try_get("status")?,
    })
}