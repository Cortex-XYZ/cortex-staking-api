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

pub async fn list_organizations(
    db: &PgPool,
) -> Result<Vec<OrganizationRecord>, sqlx::Error> {
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