use actix_web::{test, App};
use document_automation::handlers;

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(App::new().configure(handlers::health::config)).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_api_routes() {
    let app = test::init_service(
        App::new()
            .configure(handlers::auth::config)
            .configure(handlers::health::config),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/docs").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().as_u16(), 401); // Should be unauthorized
}
