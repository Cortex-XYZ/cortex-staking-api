use crate::pagination::DbPagination;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: String,
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub social_provider: Option<String>,
    pub social_provider_user_id: Option<String>,
    pub status: String,
    pub key_limit: i32,
    pub rate_limit_tier: String,
}

#[derive(Debug, Clone)]
pub struct PaginatedUsers {
    pub items: Vec<UserRecord>,
    pub total_items: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateUserInput {
    pub email: Option<String>,
    pub wallet_address: Option<String>,
    pub status: Option<String>,
    pub key_limit: Option<i32>,
    pub rate_limit_tier: Option<String>,
}

pub async fn list_users(
    db: &PgPool,
    pagination: DbPagination,
) -> Result<PaginatedUsers, sqlx::Error> {
    let total_items: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM users
        "#,
    )
    .fetch_one(db)
    .await?;

    let sort_column = match pagination.sort_column.as_str() {
        "email" => "email",
        "wallet_address" => "wallet_address",
        "status" => "status",
        "key_limit" => "key_limit",
        "rate_limit_tier" => "rate_limit_tier",
        "created_at" => "created_at",
        _ => "created_at",
    };

    let sort_direction = match pagination.sort_direction.as_str() {
        "asc" => "ASC",
        _ => "DESC",
    };

    let query = format!(
        r#"
        SELECT
            id::text,
            email,
            wallet_address,
            social_provider,
            social_provider_user_id,
            status,
            key_limit,
            rate_limit_tier
        FROM users
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
        .map(row_to_user_record)
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(PaginatedUsers { items, total_items })
}

pub async fn get_user_by_id(
    db: &PgPool,
    user_id: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
            id::text,
            email,
            wallet_address,
            social_provider,
            social_provider_user_id,
            status,
            key_limit,
            rate_limit_tier
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await?;

    row.map(row_to_user_record).transpose()
}

pub async fn update_user(
    db: &PgPool,
    user_id: &str,
    input: UpdateUserInput,
) -> Result<Option<UserRecord>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE users
        SET
            email = COALESCE($2, email),
            wallet_address = COALESCE($3, wallet_address),
            status = COALESCE($4, status),
            key_limit = COALESCE($5, key_limit),
            rate_limit_tier = COALESCE($6, rate_limit_tier),
            updated_at = now()
        WHERE id = $1::uuid
        RETURNING
            id::text,
            email,
            wallet_address,
            social_provider,
            social_provider_user_id,
            status,
            key_limit,
            rate_limit_tier
        "#,
    )
    .bind(user_id)
    .bind(input.email)
    .bind(input.wallet_address)
    .bind(input.status)
    .bind(input.key_limit)
    .bind(input.rate_limit_tier)
    .fetch_optional(db)
    .await?;

    row.map(row_to_user_record).transpose()
}

pub async fn soft_delete_user(
    db: &PgPool,
    user_id: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    update_user(
        db,
        user_id,
        UpdateUserInput {
            email: None,
            wallet_address: None,
            status: Some("deleted".to_string()),
            key_limit: None,
            rate_limit_tier: None,
        },
    )
    .await
}

fn row_to_user_record(row: sqlx::postgres::PgRow) -> Result<UserRecord, sqlx::Error> {
    Ok(UserRecord {
        id: row.try_get("id")?,
        email: row.try_get("email")?,
        wallet_address: row.try_get("wallet_address")?,
        social_provider: row.try_get("social_provider")?,
        social_provider_user_id: row.try_get("social_provider_user_id")?,
        status: row.try_get("status")?,
        key_limit: row.try_get("key_limit")?,
        rate_limit_tier: row.try_get("rate_limit_tier")?,
    })
}