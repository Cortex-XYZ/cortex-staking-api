use crate::pagination::DbPagination;
use serde_json::Value;
use sqlx::{PgPool, Row};

#[derive(Debug, Clone)]
pub struct AuditLogRecord {
    pub id: String,
    pub actor_api_key_id: Option<String>,
    pub actor_organization_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub request_id: Option<String>,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PaginatedAuditLogs {
    pub items: Vec<AuditLogRecord>,
    pub total_items: i64,
}

#[derive(Debug, Clone)]
pub struct CreateAuditLogInput {
    pub actor_api_key_id: Option<String>,
    pub actor_organization_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub request_id: Option<String>,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
}

pub async fn create_audit_log(
    db: &PgPool,
    input: CreateAuditLogInput,
) -> Result<AuditLogRecord, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO audit_logs (
            actor_api_key_id,
            actor_organization_id,
            action,
            resource_type,
            resource_id,
            ip_address,
            request_id,
            old_values,
            new_values
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3,
            $4,
            $5::uuid,
            $6,
            $7,
            $8,
            $9
        )
        RETURNING
            id::text,
            actor_api_key_id::text,
            actor_organization_id::text,
            action,
            resource_type,
            resource_id::text,
            ip_address,
            request_id,
            old_values,
            new_values,
            created_at
        "#,
    )
    .bind(input.actor_api_key_id)
    .bind(input.actor_organization_id)
    .bind(input.action)
    .bind(input.resource_type)
    .bind(input.resource_id)
    .bind(input.ip_address)
    .bind(input.request_id)
    .bind(input.old_values)
    .bind(input.new_values)
    .fetch_one(db)
    .await?;

    row_to_audit_log(row)
}

pub async fn list_audit_logs(
    db: &PgPool,
    pagination: DbPagination,
) -> Result<PaginatedAuditLogs, sqlx::Error> {
    let total_items: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM audit_logs
        "#,
    )
    .fetch_one(db)
    .await?;

    let sort_column = match pagination.sort_column.as_str() {
        "action" => "action",
        "resource_type" => "resource_type",
        "request_id" => "request_id",
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
            actor_api_key_id::text,
            actor_organization_id::text,
            action,
            resource_type,
            resource_id::text,
            ip_address,
            request_id,
            old_values,
            new_values,
            created_at
        FROM audit_logs
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
        .map(row_to_audit_log)
        .collect::<Result<Vec<_>, sqlx::Error>>()?;

    Ok(PaginatedAuditLogs { items, total_items })
}

fn row_to_audit_log(row: sqlx::postgres::PgRow) -> Result<AuditLogRecord, sqlx::Error> {
    Ok(AuditLogRecord {
        id: row.try_get("id")?,
        actor_api_key_id: row.try_get("actor_api_key_id")?,
        actor_organization_id: row.try_get("actor_organization_id")?,
        action: row.try_get("action")?,
        resource_type: row.try_get("resource_type")?,
        resource_id: row.try_get("resource_id")?,
        ip_address: row.try_get("ip_address")?,
        request_id: row.try_get("request_id")?,
        old_values: row.try_get("old_values")?,
        new_values: row.try_get("new_values")?,
        created_at: row.try_get("created_at")?,
    })
}