use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    sync::Arc,
    time::{Duration, Instant},
};
use redis::{aio::ConnectionManager, AsyncCommands};
use tracing::{error, warn};

use crate::state::AppState;
use crate::error::ApiError;

const RATE_LIMIT_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local current = redis.call('INCR', key)
if current == 1 then
    redis.call('EXPIRE', key, window)
end
return {current, redis.call('TTL', key)}
"#;

pub struct RateLimitMiddleware {
    state: Arc<AppState>,
    redis: ConnectionManager,
}

impl RateLimitMiddleware {
    pub async fn new(state: Arc<AppState>) -> Result<Self, redis::RedisError> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://redis:6379".to_string());
        
        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;
        
        Ok(Self { state, redis })
    }
    
    async fn get_rate_limit(&self, tenant_id: &str) -> Result<(u32, u32), redis::RedisError> {
        // Get rate limit based on tenant tier
        let (requests_per_second, burst) = match self.state.get_tenant_tier(tenant_id) {
            Some(tier) => match tier.as_str() {
                "premium" => (5000, 1000),
                "standard" => (1000, 200),
                _ => (100, 20),
            },
            None => (50, 10), // Default tier
        };
        
        Ok((requests_per_second, burst))
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            state: self.state.clone(),
            redis: self.redis.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    state: Arc<AppState>,
    redis: ConnectionManager,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        // Skip rate limiting for health check and metrics endpoints
        if req.path().starts_with("/health") || req.path().starts_with("/metrics") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let start_time = Instant::now();
        let state = self.state.clone();
        let service = self.service.clone();
        let mut redis = self.redis.clone();

        Box::pin(async move {
            // Get rate limit key (tenant ID or IP address)
            let rate_limit_key = if let Some(claims) = req.extensions().get::<crate::middleware::auth::Claims>() {
                format!("ratelimit:tenant:{}", claims.tenant_id)
            } else {
                format!("ratelimit:ip:{}", 
                    req.connection_info()
                        .peer_addr()
                        .unwrap_or("unknown")
                )
            };

            // Get rate limits for the tenant
            let (requests_per_second, burst) = state.get_rate_limit(&rate_limit_key).await
                .map_err(|e| {
                    error!("Failed to get rate limit: {}", e);
                    ApiError::InternalError("Rate limit check failed".into())
                })?;

            // Check rate limit using Redis
            let script = redis::Script::new(RATE_LIMIT_SCRIPT);
            let (current, ttl): (i64, i64) = script
                .key(&rate_limit_key)
                .arg(requests_per_second)
                .arg(1) // 1-second window
                .invoke_async(&mut redis)
                .await
                .map_err(|e| {
                    error!("Redis rate limit check failed: {}", e);
                    ApiError::InternalError("Rate limit check failed".into())
                })?;

            if current > requests_per_second as i64 {
                // Record rate limit metric
                state.metrics.rate_limited_requests.inc();
                
                warn!(
                    "Rate limit exceeded for {}: {} requests in last second",
                    rate_limit_key, current
                );
                
                return Err(ApiError::RateLimitExceeded(ttl as u64).into());
            }

            // Call the next service
            let response = service.call(req).await?;
            
            // Record metrics
            let duration = start_time.elapsed().as_secs_f64();
            state.metrics.requests_duration.observe(duration);
            state.metrics.requests_total.inc();
            
            Ok(response)
        })
    }
} 