use actix_web::{http::header, test, web, App};
use document_automation::middleware::SecurityHeaders;

#[actix_web::test]
async fn test_security_headers() {
    let app = test::init_service(
        App::new()
            .wrap(SecurityHeaders)
            .route("/test", web::get().to(|| async { "OK" })),
    )
    .await;

    let req = test::TestRequest::get().uri("/test").to_request();
    let response = test::call_service(&app, req).await;

    assert!(response.headers().contains_key(header::X_FRAME_OPTIONS));
    assert!(response
        .headers()
        .contains_key(header::X_CONTENT_TYPE_OPTIONS));
    assert!(response
        .headers()
        .contains_key(header::STRICT_TRANSPORT_SECURITY));
    assert!(response.headers().contains_key("x-xss-protection"));
    assert!(response.headers().contains_key("content-security-policy"));
}
