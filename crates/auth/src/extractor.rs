use actix_web::error::ErrorForbidden;

use crate::model::{AuthContext, Scope};

pub fn require_cortex_admin(auth: &AuthContext) -> Result<(), actix_web::Error> {
    if auth.is_cortex_admin() {
        Ok(())
    } else {
        Err(ErrorForbidden("Cortex admin key required"))
    }
}

pub fn require_scope(auth: &AuthContext, scope: Scope) -> Result<(), actix_web::Error> {
    if auth.has_scope(scope) {
        Ok(())
    } else {
        Err(ErrorForbidden("missing required scope"))
    }
}
