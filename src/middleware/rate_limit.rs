use crate::error::AppError;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct RateLimiter {
    requests_per_second: f64,
    burst: f64,
    state: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64, burst: f64) -> Self {
        RateLimiter {
            requests_per_second,
            burst,
            state: Arc::new(Mutex::new(HashMap::new())),
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
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            requests_per_second: self.requests_per_second,
            burst: self.burst,
            state: self.state.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    requests_per_second: f64,
    burst: f64,
    state: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
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
        let ip = req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let now = Instant::now();
        let mut state = self.state.lock().unwrap();
        let timestamps = state.entry(ip).or_insert_with(Vec::new);

        // Remove old timestamps
        timestamps.retain(|&ts| now.duration_since(ts).as_secs_f64() <= 1.0);

        if timestamps.len() as f64 <= self.burst {
            timestamps.push(now);
            let fut = self.service.call(req);
            Box::pin(async move {
                fut.await
            })
        } else {
            Box::pin(async move {
                Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"))
            })
        }
    }
} 