use crate::routes::admin::{
    __path_admin_health,
    api_keys::{
        __path_create_api_key, __path_list_api_keys, __path_revoke_api_key, ApiKeyResponse,
        CreateApiKeyRequest, CreateApiKeyResponse,
    },
    organizations::{
        __path_create_organization, __path_list_organizations, CreateOrganizationRequest,
        OrganizationResponse,
    },
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        admin_health,
        create_api_key,
        list_api_keys,
        revoke_api_key,
        create_organization,
        list_organizations,
    ),
    components(
        schemas(
            CreateOrganizationRequest,
            OrganizationResponse,
            CreateApiKeyRequest,
            CreateApiKeyResponse,
            ApiKeyResponse,
        )
    ),
    tags(
        (name = "admin", description = "Cortex admin routes")
    )
)]
pub struct AdminDoc;
