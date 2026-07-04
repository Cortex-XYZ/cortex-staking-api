use crate::config::Config;
use cortex_services::rate_limiter::RateLimiter;
use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub http_client: Client,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(
        config: Config,
        db: PgPool,
        http_client: Client,
    ) -> Self {
        Self {
            config,
            db,
            http_client,
            rate_limiter: RateLimiter::new(),
        }
    }
}