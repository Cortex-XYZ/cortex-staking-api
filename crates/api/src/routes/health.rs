use actix_web::{get, web, HttpRequest, HttpResponse, Responder};

use crate::{
    extractors::request_id::get_request_id,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/healthz",
    tag = "health",
    responses(
        (status = 200, description = "API is healthy")
    )
)]
#[get("/healthz")]
async fn healthz(
    req: HttpRequest,
) -> impl Responder {
    let request_id = get_request_id(&req);

    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "cortex-staking-api",
        "version": env!("CARGO_PKG_VERSION"),
        "request_id": request_id,
    }))
}

#[utoipa::path(
    get,
    path = "/readyz",
    tag = "health",
    responses(
        (status = 200, description = "API dependencies are ready"),
        (status = 503, description = "One or more dependencies are unavailable")
    )
)]
#[get("/readyz")]
pub async fn readyz(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let request_id = get_request_id(&req);

    let db_result = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await;

    match db_result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "service": "cortex-staking-api",
            "database": "ok",
            "request_id": request_id,
        })),

        Err(_) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "service": "cortex-staking-api",
            "database": "unavailable",
            "request_id": request_id,
        })),
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(healthz)
        .service(readyz);
}