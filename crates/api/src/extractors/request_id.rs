use actix_web::HttpMessage;

use crate::middleware::request_id::RequestId;

pub fn get_request_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions()
        .get::<RequestId>()
        .map(|request_id| request_id.0.clone())
}