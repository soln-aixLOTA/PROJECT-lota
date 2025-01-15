use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorTooManyRequests,
    http::header,
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use parking_lot::RwLock;

// Token bucket for rate limiting
#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    capacity: f64,
    rate: f64,
}

impl TokenBucket {
    fn new(capacity: f64, rate: f64) -> Self {
        Self {
            tokens: capacity,
            last_update: Instant::now(),
            capacity,
            rate,
        }
    }

    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_update = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn time_to_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64((1.0 - self.tokens) / self.rate)
        }
    }
}

// Rate limiter state
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    capacity: f64,
    rate: f64,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            capacity: requests_per_minute as f64,
            rate: requests_per_minute as f64 / 60.0,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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
            buckets: self.buckets.clone(),
            capacity: self.capacity,
            rate: self.rate,
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    capacity: f64,
    rate: f64,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let mut buckets = self.buckets.write();
        let bucket = buckets
            .entry(ip.clone())
            .or_insert_with(|| TokenBucket::new(self.capacity, self.rate));

        if bucket.try_consume() {
            let fut = self.service.call(req);
            Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        } else {
            let wait_time = bucket.time_to_next_token();
            drop(buckets);

            Box::pin(async move {
                tokio::time::sleep(wait_time).await;
                Err(ErrorTooManyRequests("Rate limit exceeded").into())
            })
        }
    }
}
