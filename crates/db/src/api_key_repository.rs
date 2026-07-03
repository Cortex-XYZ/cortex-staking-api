use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: String,
    pub owner_type: String,
    pub organization_id: Option<String>,
    pub user_id: Option<String>,
    pub name: String,
    pub key_prefix: String,
    pub status: String,
    pub rate_limit_per_minute: i32,
}

#[derive(Debug, Clone)]
pub struct CreateOrganizationApiKeyInput {
    pub organization_id: String,
    pub name: String,
    pub key_prefix: String,
    pub token: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
}

pub async fn create_organization_api_key(
    db: &PgPool,
    input: CreateOrganizationApiKeyInput,
) -> Result<ApiKeyRecord, sqlx::Error> {
    let mut tx = db.begin().await?;

    let row = sqlx::query(
        r#"
        INSERT INTO api_keys (
            owner_type,
            organization_id,
            name,
            key_prefix,
            key_hash,
            rate_limit_per_minute
        )
        VALUES (
            'organization',
            $1::uuid,
            $2,
            $3,
            encode(digest($4, 'sha256'), 'hex'),
            $5
        )
        RETURNING
            id::text,
            owner_type,
            organization_id::text,
            user_id::text,
            name,
            key_prefix,
            status,
            rate_limit_per_minute
        "#,
    )
    .bind(&input.organization_id)
    .bind(&input.name)
    .bind(&input.key_prefix)
    .bind(&input.token)
    .bind(input.rate_limit_per_minute)
    .fetch_one(&mut *tx)
    .await?;

    let api_key_id: String = row.try_get("id")?;

    for scope in input.scopes {
        sqlx::query(
            r#"
            INSERT INTO api_key_scopes (api_key_id, scope)
            VALUES ($1::uuid, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&api_key_id)
        .bind(scope)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(ApiKeyRecord {
        id: row.try_get("id")?,
        owner_type: row.try_get("owner_type")?,
        organization_id: row.try_get("organization_id")?,
        user_id: row.try_get("user_id")?,
        name: row.try_get("name")?,
        key_prefix: row.try_get("key_prefix")?,
        status: row.try_get("status")?,
        rate_limit_per_minute: row.try_get("rate_limit_per_minute")?,
    })
}

pub async fn list_api_keys(db: &PgPool) -> Result<Vec<ApiKeyRecord>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            id::text,
            owner_type,
            organization_id::text,
            user_id::text,
            name,
            key_prefix,
            status,
            rate_limit_per_minute
        FROM api_keys
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ApiKeyRecord {
                id: row.try_get("id")?,
                owner_type: row.try_get("owner_type")?,
                organization_id: row.try_get("organization_id")?,
                user_id: row.try_get("user_id")?,
                name: row.try_get("name")?,
                key_prefix: row.try_get("key_prefix")?,
                status: row.try_get("status")?,
                rate_limit_per_minute: row.try_get("rate_limit_per_minute")?,
            })
        })
        .collect()
}

pub async fn revoke_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    soft_delete_api_key(db, api_key_id).await
}

pub async fn get_api_key_by_id(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text,
            owner_type,
            organization_id::text,
            user_id::text,
            name,
            key_prefix,
            status,
            rate_limit_per_minute
        FROM api_keys
        WHERE id = $1::uuid
        "#,
    )
    .bind(api_key_id)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(ApiKeyRecord {
            id: row.try_get("id")?,
            owner_type: row.try_get("owner_type")?,
            organization_id: row.try_get("organization_id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            key_prefix: row.try_get("key_prefix")?,
            status: row.try_get("status")?,
            rate_limit_per_minute: row.try_get("rate_limit_per_minute")?,
        })
    })
    .transpose()
}

pub async fn soft_delete_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE api_keys
        SET
            status = 'revoked',
            revoked_at = COALESCE(revoked_at, now()),
            updated_at = now()
        WHERE id = $1::uuid
        RETURNING
            id::text,
            owner_type,
            organization_id::text,
            user_id::text,
            name,
            key_prefix,
            status,
            rate_limit_per_minute
        "#,
    )
    .bind(api_key_id)
    .fetch_optional(db)
    .await?;

    row.map(|row| {
        Ok(ApiKeyRecord {
            id: row.try_get("id")?,
            owner_type: row.try_get("owner_type")?,
            organization_id: row.try_get("organization_id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            key_prefix: row.try_get("key_prefix")?,
            status: row.try_get("status")?,
            rate_limit_per_minute: row.try_get("rate_limit_per_minute")?,
        })
    })
    .transpose()
}

pub async fn get_api_key_scopes(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT scope
        FROM api_key_scopes
        WHERE api_key_id = $1::uuid
        ORDER BY scope
        "#,
    )
    .bind(api_key_id)
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| row.try_get("scope"))
        .collect()
}