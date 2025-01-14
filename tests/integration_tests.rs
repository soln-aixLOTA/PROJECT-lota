use actix_web::{test, web, App};
use document_automation::{
    handlers,
    middleware::rate_limit::RateLimiter,
};
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
async fn test_health_check() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/document_automation_test")
        .await
        .expect("Failed to connect to Postgres");

    let app = test_app(pool).await;
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_rate_limiting() {
    let pool = PgPool::connect("postgres://postgres:postgres@localhost:5432/document_automation_test")
        .await
        .expect("Failed to connect to Postgres");

    let app = test_app(pool).await;

    // Make 101 requests (exceeding the 100 per second limit)
    for _ in 0..101 {
        let req = test::TestRequest::get().uri("/api/v1/health").to_request();
        let resp = test::call_service(&app, req).await;

        if resp.status().as_u16() == 429 {
            // Rate limit exceeded as expected
            return;
        }
    }

    panic!("Rate limiting did not trigger after 101 requests");
}
