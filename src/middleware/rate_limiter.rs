use axum::{
    middleware::Next,
    response::Response,
    http::{Request, HeaderMap},
    extract::Extension,
};
use governor::{DefaultKeyedRateLimiter, Quota, clock::DefaultClock};
use std::{sync::Arc, num::NonZeroU32};
use tokio::sync::Mutex;

type RateLimiter = Arc<Mutex<DefaultKeyedRateLimiter<String, DefaultClock>>>;

pub async fn rate_limiter<B>(
    Extension(limiter): Extension<RateLimiter>,
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError> {
    let ip = headers
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    let mut limiter = limiter.lock().await;
    
    if limiter.check_key(&ip.to_string()).is_err() {
        return Err(ApiError::RateLimitExceeded);
    }

    Ok(next.run(request).await)
}

pub fn create_rate_limiter() -> RateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
    Arc::new(Mutex::new(DefaultKeyedRateLimiter::keyed(quota)))
} 