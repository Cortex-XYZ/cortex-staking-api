use crate::state::AppState;
use actix_web::{HttpResponse, Responder, get, web};

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "API is healthy"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "Service Unavailable"),
        (status = 504, description = "Gateway Timeout")
    )
)]
#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "cortex-staking-api",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    responses(
        (status = 200, description = "API dependencies are ready"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error"),
        (status = 503, description = "One or more dependencies are unavailable"),
        (status = 504, description = "Gateway Timeout")
    )
)]
#[get("/readyz")]
pub async fn readyz(state: web::Data<AppState>) -> impl Responder {
    let db_result = sqlx::query("SELECT 1").execute(&state.db).await;

    match db_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "cortex-staking-api",
            "database": "ok"
        })),

        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "service": "cortex-staking-api",
            "database": "unavailable"
        })),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz).service(readyz);
}
