use crate::routes::health::{
    __path_healthz, 
    __path_readyz,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(healthz, readyz),
    tags((name = "health", description = "Service health routes"))
)]
pub struct HealthDoc;
