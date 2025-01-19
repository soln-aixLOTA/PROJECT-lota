use actix_web::{test, App};
use document_automation::handlers;

#[actix_web::test]
async fn test_health_handler() {
    let app = test::init_service(App::new().configure(handlers::health::config)).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}
