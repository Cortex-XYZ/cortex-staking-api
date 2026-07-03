pub mod api_keys;
pub mod organizations;

use actix_web::{HttpResponse, Responder, get, web};
use cortex_auth::extractor::require_cortex_admin;

use crate::extractors::auth::Authenticated;

#[utoipa::path(
    get,
    path = "/admin/health",
    tag = "admin",
    responses(
        (status = 200, description = "Admin route group is healthy"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Not Found"),
        (status = 405, description = "Method Not Allowed"),
        (status = 429, description = "Too Many Requests"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable"),
        (status = 504, description = "Gateway Timeout")
    )
)]
#[get("/health")]
pub async fn admin_health(auth: Authenticated) -> actix_web::Result<impl Responder> {
    require_cortex_admin(&auth.0)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "scope": "admin"
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(admin_health)
        .configure(organizations::configure)
        .configure(api_keys::configure);
}
