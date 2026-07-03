use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct OrganizationRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
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

    Ok(OrganizationRecord {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        kind: row.try_get("kind")?,
        status: row.try_get("status")?,
    })
}

pub async fn list_organizations(db: &PgPool) -> Result<Vec<OrganizationRecord>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id::text, name, kind, status
        FROM organizations
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(OrganizationRecord {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                kind: row.try_get("kind")?,
                status: row.try_get("status")?,
            })
        })
        .collect()
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

    row.map(|row| {
        Ok(OrganizationRecord {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            kind: row.try_get("kind")?,
            status: row.try_get("status")?,
        })
    })
    .transpose()
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

    row.map(|row| {
        Ok(OrganizationRecord {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            kind: row.try_get("kind")?,
            status: row.try_get("status")?,
        })
    })
    .transpose()
}

pub async fn soft_delete_organization(
    db: &PgPool,
    organization_id: &str,
) -> Result<Option<OrganizationRecord>, sqlx::Error> {
    update_organization(db, organization_id, None, Some("deleted")).await
}