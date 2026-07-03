use cortex_db::audit_repository::{self, AuditLogRecord};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct RecordAuditLogInput {
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

pub async fn record_admin_action(
    db: &PgPool,
    input: RecordAuditLogInput,
) -> Result<AuditLogRecord, sqlx::Error> {
    audit_repository::create_audit_log(
        db,
        audit_repository::CreateAuditLogInput {
            actor_api_key_id: input.actor_api_key_id,
            actor_organization_id: input.actor_organization_id,
            action: input.action,
            resource_type: input.resource_type,
            resource_id: input.resource_id,
            ip_address: input.ip_address,
            request_id: input.request_id,
            old_values: input.old_values,
            new_values: input.new_values,
        },
    )
    .await
}

pub async fn list_audit_logs(
    db: &PgPool,
) -> Result<Vec<AuditLogRecord>, sqlx::Error> {
    audit_repository::list_audit_logs(db).await
}