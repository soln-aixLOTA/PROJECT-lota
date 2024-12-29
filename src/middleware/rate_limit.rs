use std::future::{ready, Ready};
use std::sync::Arc;
use std::time::{Duration, Instant};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    Error,
};
use futures::future::LocalBoxFuture;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use tracing::warn;

/// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    capacity: f64,
    refill_rate: f64,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            tokens: capacity as f64,
            last_update: Instant::now(),
            capacity: capacity as f64,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        self.last_update = now;

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

/// Rate limiter configuration
#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    pub requests_per_second: f64,
    pub burst_size: u32,
    pub error_message: String,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 50,
            error_message: "Too many requests".to_string(),
        }
    }
}

/// Rate limiter state
#[derive(Debug, Default)]
struct RateLimiterState {
    buckets: HashMap<IpAddr, TokenBucket>,
}

impl RateLimiterState {
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.buckets
            .retain(|_, bucket| now.duration_since(bucket.last_update) < Duration::from_secs(3600));
    }
}

/// Rate limiter middleware
pub struct RateLimiter {
    config: RateLimiterConfig,
    state: Arc<Mutex<RateLimiterState>>,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(RateLimiterState::default())),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            config: self.config.clone(),
            state: self.state.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    config: RateLimiterConfig,
    state: Arc<Mutex<RateLimiterState>>,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .and_then(|addr| addr.parse::<IpAddr>().ok());

        let mut state = self.state.lock();

        // Periodic cleanup of old entries
        if rand::random::<f64>() < 0.01 {
            state.cleanup();
        }

        if let Some(ip) = ip {
            let bucket = state.buckets.entry(ip).or_insert_with(|| {
                TokenBucket::new(self.config.burst_size, self.config.requests_per_second)
            });

            if !bucket.try_consume(1.0) {
                warn!("Rate limit exceeded for IP: {}", ip);
                let error_msg = self.config.error_message.clone();
                return Box::pin(async move { Err(Error::from(ErrorTooManyRequests(error_msg))) });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{self, TestRequest};
    use actix_web::{web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[actix_web::test]
    async fn test_rate_limiter() {
        let config = RateLimiterConfig {
            requests_per_second: 2.0,
            burst_size: 2,
            error_message: "Too many requests".to_string(),
        };

        let app = test::init_service(
            App::new()
                .wrap(RateLimiter::new(config))
                .route("/", web::get().to(test_handler)),
        )
        .await;

        // First request should succeed
        let req = TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Second request should succeed
        let req = TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Third request should fail with 429
        let req = TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 429);
    }
}
