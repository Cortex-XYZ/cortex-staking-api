use actix_web::{FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized, web};
use cortex_auth::model::AuthContext;
use futures_util::future::LocalBoxFuture;

use crate::state::AppState;

pub struct Authenticated(pub AuthContext);

impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let state = req.app_data::<web::Data<AppState>>().cloned();

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::to_string);

        Box::pin(async move {
            let Some(state) = state else {
                return Err(ErrorUnauthorized("missing app state"));
            };

            let Some(token) = token else {
                return Err(ErrorUnauthorized("missing or invalid Authorization header"));
            };

            let auth = cortex_db::auth_repository::authenticate_api_key(&state.db, &token)
                .await
                .map_err(|_| ErrorUnauthorized("failed to authenticate API key"))?;

            match auth {
                Some(auth) => Ok(Authenticated(auth)),
                None => Err(ErrorUnauthorized("invalid API key")),
            }
        })
    }
}
