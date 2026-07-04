use actix_web::{http::StatusCode, HttpResponse};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
}

pub fn error_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
    request_id: Option<String>,
) -> HttpResponse {
    HttpResponse::build(status).json(ErrorEnvelope {
        error: ErrorBody {
            code: code.into(),
            message: message.into(),
            request_id,
        },
    })
}

pub fn not_found(
    code: impl Into<String>,
    message: impl Into<String>,
    request_id: Option<String>,
) -> HttpResponse {
    error_response(StatusCode::NOT_FOUND, code, message, request_id)
}

pub fn internal_error(request_id: Option<String>) -> HttpResponse {
    error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal_server_error",
        "Internal server error",
        request_id,
    )
}