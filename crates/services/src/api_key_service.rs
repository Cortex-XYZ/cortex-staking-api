use cortex_auth::api_key::generate_api_key;
use cortex_db::api_key_repository::{
    self, ApiKeyRecord, CreateOrganizationApiKeyInput,
};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct CreateOrganizationApiKeyServiceInput {
    pub organization_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
}

#[derive(Debug, Clone)]
pub struct CreatedApiKey {
    pub api_key: ApiKeyRecord,
    pub token: String,
}

pub async fn create_organization_api_key(
    db: &PgPool,
    input: CreateOrganizationApiKeyServiceInput,
) -> Result<CreatedApiKey, sqlx::Error> {
    let generated = generate_api_key("partner");

    let api_key = api_key_repository::create_organization_api_key(
        db,
        CreateOrganizationApiKeyInput {
            organization_id: input.organization_id,
            name: input.name,
            key_prefix: generated.key_prefix,
            token: generated.token.clone(),
            scopes: input.scopes,
            rate_limit_per_minute: input.rate_limit_per_minute,
        },
    )
    .await?;

    Ok(CreatedApiKey {
        api_key,
        token: generated.token,
    })
}

pub async fn list_api_keys(db: &PgPool) -> Result<Vec<ApiKeyRecord>, sqlx::Error> {
    api_key_repository::list_api_keys(db).await
}

pub async fn revoke_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    api_key_repository::revoke_api_key(db, api_key_id).await
}