use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use dashmap::DashMap;
use futures::future::{ok, Future, Ready};
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_json::json;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{error, warn};
use uuid::Uuid;

lazy_static! {
    static ref RATE_LIMIT_HITS: CounterVec = register_counter_vec!(
        "api_rate_limit_hits",
        "API rate limit hits by tenant and tier",
        &["tenant_id", "tier"]
    )
    .unwrap();

    static ref RATE_LIMITS: DashMap<String, RateLimit> = {
        let mut m = DashMap::new();

        // Define rate limits for different tiers
        m.insert(
            "free".to_string(),
            RateLimit {
                requests_per_second: 2,
                burst_size: 5,
            },
        );

        m.insert(
            "basic".to_string(),
            RateLimit {
                requests_per_second: 10,
                burst_size: 20,
            },
        );

        m.insert(
            "premium".to_string(),
            RateLimit {
                requests_per_second: 50,
                burst_size: 100,
            },
        );

        m
    };
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    rate: f64,
    capacity: f64,
}

impl TokenBucket {
    fn new(rate: f64, capacity: f64) -> Self {
        Self {
            tokens: capacity,
            last_update: Instant::now(),
            rate,
            capacity,
        }
    }

    fn try_consume(&mut self, tokens: f64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_update = now;

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

pub struct RateLimitMiddleware {
    buckets: Arc<DashMap<String, Arc<Mutex<TokenBucket>>>>,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
        }
    }

    fn get_rate_limit(&self, tenant_id: &str) -> RateLimit {
        // In a real application, this would look up the tenant's tier from a database
        // For now, we'll use the "basic" tier as default
        RATE_LIMITS
            .get("basic")
            .map(|r| r.clone())
            .unwrap_or_else(|| RateLimit {
                requests_per_second: 5,
                burst_size: 10,
            })
    }

    async fn check_rate_limit(
        &self,
        tenant_id: &str,
        bucket_key: &str,
    ) -> Result<(), actix_web::error::Error> {
        let rate_limit = self.get_rate_limit(tenant_id);
        let bucket = self
            .buckets
            .entry(bucket_key.to_string())
            .or_insert_with(|| {
                Arc::new(Mutex::new(TokenBucket::new(
                    rate_limit.requests_per_second as f64,
                    rate_limit.burst_size as f64,
                )))
            })
            .clone();

        let mut bucket = bucket.lock().await;
        if !bucket.try_consume(1.0) {
            RATE_LIMIT_HITS
                .with_label_values(&[tenant_id, "basic"])
                .inc();

            warn!(
                "Rate limit exceeded for tenant {} with key {}",
                tenant_id, bucket_key
            );

            return Err(actix_web::error::ErrorTooManyRequests(json!({
                "error": "rate_limit_exceeded",
                "message": "Too many requests",
                "retry_after": (1.0 / rate_limit.requests_per_second as f64).ceil() as u32
            })));
        }

        Ok(())
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            buckets: self.buckets.clone(),
        })
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    buckets: Arc<DashMap<String, Arc<Mutex<TokenBucket>>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let tenant_id = req
            .extensions()
            .get::<Uuid>()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "anonymous".to_string());

        let bucket_key = format!("{}:{}", tenant_id, req.path());
        let buckets = self.buckets.clone();
        let middleware = RateLimitMiddleware { buckets };

        Box::pin(async move {
            middleware.check_rate_limit(&tenant_id, &bucket_key).await?;

            let fut = self.service.call(req);
            fut.await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::{web, App, HttpResponse};
    use std::time::Duration;
    use tokio::time::sleep;

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[actix_web::test]
    async fn test_rate_limiting() {
        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new())
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // First request should succeed
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Immediate second request should also succeed (due to burst)
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Third immediate request should fail
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 429);

        // Wait for token bucket to refill
        sleep(Duration::from_secs(1)).await;

        // Request should succeed again
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
