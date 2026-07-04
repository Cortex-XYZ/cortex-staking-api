use actix_web::web;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

use cortex_staking_api::{
    app, 
    config::Config, 
    middleware::request_id::RequestIdMiddleware,
    state::AppState
};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt().init();

    let config = Config::from_env();
    let bind_address = config.bind_address.clone();

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&config.database_url)?;

    let http_client = Client::builder()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(30))
        .build()?;

    let state = web::Data::new(AppState::new(config, db, http_client));

    tracing::info!("starting cortex staking api on {}", bind_address);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
        .wrap(RequestIdMiddleware)
            .app_data(state.clone())
            .configure(app::configure_app)
    })
    .bind(bind_address)?
    .run()
    .await?;

    Ok(())
}
