use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use dashmap::DashMap;
use futures_util::future::{ready, LocalBoxFuture, Ready};
use lazy_static::lazy_static;
use prometheus::{register_int_counter_vec, IntCounterVec};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tracing::warn;

lazy_static! {
    static ref RATE_LIMIT_HITS: IntCounterVec = register_int_counter_vec!(
        "api_gateway_rate_limit_hits_total",
        "Total number of rate limit hits",
        &["endpoint", "method", "tier"]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            requests_per_second: 10,
            burst_size: 50,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware {
    config: Arc<RateLimitConfig>,
    limits: Arc<DashMap<String, (Instant, u32)>>,
}

impl RateLimitMiddleware {
    pub fn new() -> Self {
        RateLimitMiddleware {
            config: Arc::new(RateLimitConfig::default()),
            limits: Arc::new(DashMap::new()),
        }
    }

    pub fn with_config(config: RateLimitConfig) -> Self {
        RateLimitMiddleware {
            config: Arc::new(config),
            limits: Arc::new(DashMap::new()),
        }
    }

    fn get_rate_limit_key(&self, req: &ServiceRequest) -> String {
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let path = req.path().to_string();
        let method = req.method().to_string();
        format!("{}:{}:{}", ip, method, path)
    }

    async fn check_rate_limit(&self, key: &str) -> Result<(), Error> {
        let now = Instant::now();
        let config = self.config.clone();

        let mut entry = self
            .limits
            .entry(key.to_string())
            .or_insert_with(|| (now, 0));
        let (last_reset, count) = entry.value_mut();

        if now.duration_since(*last_reset) >= Duration::from_secs(1) {
            *last_reset = now;
            *count = 1;
        } else {
            *count += 1;
            if *count > config.requests_per_second + config.burst_size {
                warn!("Rate limit exceeded for key: {}", key);
                RATE_LIMIT_HITS
                    .with_label_values(&["unknown", "unknown", "default"])
                    .inc();
                return Err(Error::from(crate::error::ApiError::new(
                    "RATE_LIMIT_EXCEEDED",
                    "Too many requests",
                )));
            }
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
        ready(Ok(RateLimitMiddlewareService {
            service,
            inner: self.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    inner: RateLimitMiddleware,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = self.inner.get_rate_limit_key(&req);
        let inner = self.inner.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            inner.check_rate_limit(&key).await?;
            fut.await
        })
    }
}
