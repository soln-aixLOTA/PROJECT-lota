use actix_web::{
    test,
    web,
    App,
};
use document_automation::{
    handlers,
    middleware::rate_limit::RateLimiter,
};
use serde_json::json;
use sqlx::PgPool;

async fn test_app(pool: PgPool) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .wrap(RateLimiter::new(100.0))
            .configure(handlers::config)
    ).await
}

#[actix_web::test]
async fn test_document_creation() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/document_automation_test")
        .await
        .expect("Failed to connect to Postgres");

    let app = test_app(pool).await;

    // Example document creation
    let create_req = test::TestRequest::post()
        .uri("/api/v1/documents")
        .set_json(json!({ "title": "test doc", "content": "Hello, world!" }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success(), "Expected document creation success");
}
