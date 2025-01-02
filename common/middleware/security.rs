use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderMap},
};
use tracing::warn;

pub async fn security_headers<B>(
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:;".parse().unwrap(),
    );

    response.headers_mut().insert(
        "X-Content-Type-Options",
        "nosniff".parse().unwrap(),
    );

    response.headers_mut().insert(
        "X-Frame-Options",
        "DENY".parse().unwrap(),
    );

    response.headers_mut().insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    response
} 