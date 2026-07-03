use actix_web::{HttpResponse, Responder, get, web};
use cortex_auth::{extractor::require_scope, model::Scope};

use crate::extractors::auth::Authenticated;
#[utoipa::path(
    get,
    path = "/monad/health",
    responses(
        (status = 200, description = "API is healthy"),
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
pub async fn monad_health(auth: Authenticated) -> actix_web::Result<impl Responder> {
    require_scope(&auth.0, Scope::Read)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "chain": "monad"
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(monad_health);
}
