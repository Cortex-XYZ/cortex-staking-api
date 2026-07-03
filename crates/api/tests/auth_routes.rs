use actix_web::{App, http::StatusCode, test, web};
use cortex_staking_api::{app, config::Config, state::AppState};
use sqlx::postgres::PgPoolOptions;

fn test_state() -> web::Data<AppState> {
    dotenvy::dotenv().ok();

    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&config.database_url)
        .expect("failed to create lazy db pool");

    let http_client = reqwest::Client::new();

    web::Data::new(AppState::new(config, db, http_client))
}

#[actix_web::test]
async fn healthz_returns_ok_without_auth() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn readyz_route_exists() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get().uri("/readyz").to_request();
    let resp = test::call_service(&app, req).await;

    assert_ne!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn admin_route_rejects_missing_key() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get().uri("/admin/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn admin_route_rejects_partner_key() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/health")
        .insert_header(("Authorization", "Bearer ctx_dev_partner.secret"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn admin_route_accepts_cortex_key() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/health")
        .insert_header(("Authorization", "Bearer ctx_dev_cortex.secret"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn monad_route_accepts_partner_key() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/monad/health")
        .insert_header(("Authorization", "Bearer ctx_dev_partner.secret"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn cortex_key_can_list_organizations() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_cortex.secret"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn partner_key_cannot_list_organizations() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_partner.secret"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn partner_key_cannot_create_organization() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_partner.secret"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"name":"Should Fail"}"#)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
async fn cortex_key_can_create_partner_organization() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let unique_name = format!(
        "Test Partner {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis()
    );

    let req = test::TestRequest::post()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_cortex.secret"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(format!(r#"{{"name":"{}"}}"#, unique_name))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[actix_web::test]
async fn created_organization_appears_in_list() {
    let app = test::init_service(
        App::new()
            .app_data(test_state())
            .configure(app::configure_app),
    )
    .await;

    let unique_name = format!(
        "List Test Partner {}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis()
    );

    let create_req = test::TestRequest::post()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_cortex.secret"))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(format!(r#"{{"name":"{}"}}"#, unique_name))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    let list_req = test::TestRequest::get()
        .uri("/admin/organizations")
        .insert_header(("Authorization", "Bearer ctx_dev_cortex.secret"))
        .to_request();

    let list_resp = test::call_service(&app, list_req).await;
    assert_eq!(list_resp.status(), StatusCode::OK);

    let body = test::read_body(list_resp).await;
    let body_text = std::str::from_utf8(&body).expect("response body should be utf8");

    assert!(body_text.contains(&unique_name));
}