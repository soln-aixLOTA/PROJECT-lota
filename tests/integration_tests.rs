use actix_web::{test, web, App};
use handlers::configure_routes;

#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/health")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}

#[actix_web::test]
async fn test_not_found_route() {
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/v1/non-existent")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
} 