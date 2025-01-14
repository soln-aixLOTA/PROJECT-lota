use actix_web::{test, web, App, http::StatusCode};
use document_automation::{
    handlers,
    middleware::rate_limit::RateLimiter,
};
use sqlx::PgPool;
use std::env;

async fn setup_test_app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .wrap(RateLimiter::new(100.0))
            .configure(handlers::config)
    ).await
}

#[actix_web::test]
async fn test_rate_limiting() {
    let app = setup_test_app().await;

    // Make multiple requests in quick succession
    for _ in 0..150 {
        let req = test::TestRequest::get()
            .uri("/api/v1/health")
            .to_request();
        let resp = test::call_service(&app, req).await;

        if resp.status() == StatusCode::TOO_MANY_REQUESTS {
            // Rate limit was hit as expected
            return;
        }
    }

    panic!("Rate limit was not enforced");
}
