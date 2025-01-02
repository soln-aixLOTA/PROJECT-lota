use super::*;
use axum::{routing::get, Router, http::Request, body::Body};
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limiting() {
    let limiter = create_rate_limiter();
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .layer(middleware::from_fn_with_state(limiter.clone(), rate_limiter));

    // First 100 requests should succeed
    for _ in 0..100 {
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // 101st request should fail
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
} 