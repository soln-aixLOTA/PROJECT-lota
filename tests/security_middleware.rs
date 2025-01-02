use axum::{
    routing::get,
    Router,
    http::{StatusCode, Request},
    body::Body,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_security_headers() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(middleware::from_fn(security_headers));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["Content-Security-Policy"],
        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;"
    );
    assert_eq!(response.headers()["X-Content-Type-Options"], "nosniff");
} 