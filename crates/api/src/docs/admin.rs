use crate::routes::admin::{
    __path_admin_health,
    organizations::{
        __path_create_organization,
        __path_list_organizations,
        CreateOrganizationRequest,
        OrganizationResponse,
    },
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        admin_health,
        create_organization,
        list_organizations,
    ),
    components(
        schemas(
            CreateOrganizationRequest,
            OrganizationResponse,
        )
    ),
    tags(
        (name = "admin", description = "Cortex admin routes")
    )
)]
pub struct AdminDoc;
