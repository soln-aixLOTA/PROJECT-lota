use actix_web::{
    http::StatusCode,
    test,
    web,
    App,
};
use document_automation::middleware::RateLimiter;

#[actix_web::test]
async fn test_rate_limiter() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimiter::new(1.0, 0.1)) // 1 request per 10 seconds
            .route("/test", web::get().to(|| async { "OK" }))
    ).await;

    // First request should succeed
    let req = test::TestRequest::get().uri("/test").to_request();
    let response = test::call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Second request should be rate limited
    let req = test::TestRequest::get().uri("/test").to_request();
    let response = test::call_service(&app, req).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
} 