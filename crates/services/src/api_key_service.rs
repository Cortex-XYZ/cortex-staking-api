use cortex_auth::api_key::generate_api_key;
use cortex_db::{
    api_key_repository::{
        self, ApiKeyRecord, CreateOrganizationApiKeyInput, PaginatedApiKeys,
    }, pagination::DbPagination,
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

pub async fn list_api_keys(
    db: &PgPool, 
    pagination: DbPagination,
) -> Result<PaginatedApiKeys, sqlx::Error> {
    api_key_repository::list_api_keys(db, pagination).await
}

pub async fn get_api_key_by_id(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    api_key_repository::get_api_key_by_id(db, api_key_id).await
}

pub async fn revoke_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    api_key_repository::revoke_api_key(db, api_key_id).await
}

pub async fn delete_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<ApiKeyRecord>, sqlx::Error> {
    api_key_repository::soft_delete_api_key(db, api_key_id).await
}

pub async fn rotate_api_key(
    db: &PgPool,
    api_key_id: &str,
) -> Result<Option<CreatedApiKey>, sqlx::Error> {
    let Some(old_key) = api_key_repository::get_api_key_by_id(db, api_key_id).await? else {
        return Ok(None);
    };

    let Some(organization_id) = old_key.organization_id.clone() else {
        return Ok(None);
    };

    let scopes = api_key_repository::get_api_key_scopes(db, api_key_id).await?;

    api_key_repository::revoke_api_key(db, api_key_id).await?;

    let generated = generate_api_key("partner");

    let new_key = api_key_repository::create_organization_api_key(
        db,
        CreateOrganizationApiKeyInput {
            organization_id,
            name: format!("{} (rotated)", old_key.name),
            key_prefix: generated.key_prefix,
            token: generated.token.clone(),
            scopes,
            rate_limit_per_minute: old_key.rate_limit_per_minute,
        },
    )
    .await?;

    Ok(Some(CreatedApiKey {
        api_key: new_key,
        token: generated.token,
    }))
}