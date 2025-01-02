use std::collections::HashMap;
use actix_web::dev::{Service, Transform};
use actix_web::{Error, HttpMessage, http::Method};
use futures::future::{ok, Ready};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use prometheus::{IntCounter, register_int_counter};
use lazy_static::lazy_static;

use crate::error::{ApiError, handle_rate_limit_error};

lazy_static! {
    static ref RATE_LIMIT_HITS: IntCounter = register_int_counter!(
        "api_gateway_rate_limit_hits_total",
        "Total number of rate limit hits",
        &["endpoint", "method", "tier"]
    ).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub requests_per_window: u32,
    pub window_seconds: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointLimit {
    pub path: String,
    pub method: Option<String>,  // None means all methods
    pub rate_limit: RateLimitInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub name: String,
    pub base_rate_limit: RateLimitInfo,
    pub endpoint_limits: Vec<EndpointLimit>,
}

#[derive(Clone)]
pub struct RateLimitConfig {
    tier_limits: Arc<HashMap<String, TierConfig>>,
    usage: Arc<RwLock<HashMap<String, HashMap<String, (Instant, u32)>>>>,  // tenant_id -> (endpoint -> (window_start, count))
}

impl RateLimitConfig {
    pub fn new() -> Self {
        Self {
            tier_limits: Arc::new(HashMap::new()),
            usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_tier(mut self, tier: &str, base_requests: u32, window_seconds: u32) -> Self {
        let config = TierConfig {
            name: tier.to_string(),
            base_rate_limit: RateLimitInfo {
                requests_per_window: base_requests,
                window_seconds,
                burst_size: base_requests / 10, // Allow 10% burst
            },
            endpoint_limits: Vec::new(),
        };

        Arc::get_mut(&mut self.tier_limits)
            .expect("Arc should be unique at this point")
            .insert(tier.to_string(), config);
        
        self
    }

    pub fn with_endpoint_limit(
        mut self,
        tier: &str,
        path: &str,
        method: Option<&str>,
        requests: u32,
        window_seconds: u32
    ) -> Self {
        let tier_limits = Arc::get_mut(&mut self.tier_limits)
            .expect("Arc should be unique at this point");
        
        if let Some(tier_config) = tier_limits.get_mut(tier) {
            tier_config.endpoint_limits.push(EndpointLimit {
                path: path.to_string(),
                method: method.map(String::from),
                rate_limit: RateLimitInfo {
                    requests_per_window: requests,
                    window_seconds,
                    burst_size: requests / 10,
                },
            });
        }
        
        self
    }

    pub fn into_middleware(self) -> RateLimitMiddleware {
        RateLimitMiddleware { config: self }
    }

    async fn check_rate_limit(
        &self,
        tenant_id: &str,
        tier: &str,
        path: &str,
        method: &Method,
    ) -> Result<RateLimitStatus, ApiError> {
        let mut usage = self.usage.write().await;
        let now = Instant::now();
        
        let tier_config = self.tier_limits
            .get(tier)
            .ok_or_else(|| handle_rate_limit_error(
                format!("Invalid subscription tier: {}", tier),
                Some(json!({ "tier": tier }))
            ))?;

        // If it's unlimited (enterprise tier)
        if tier_config.base_rate_limit.requests_per_window == 0 {
            return Ok(RateLimitStatus::Allowed { remaining: u32::MAX });
        }

        // Find endpoint-specific limit if it exists
        let limit_info = tier_config.endpoint_limits
            .iter()
            .find(|limit| {
                limit.path == path && (
                    limit.method.is_none() || 
                    limit.method.as_ref().map(|m| m == method.as_str()).unwrap_or(false)
                )
            })
            .map(|limit| &limit.rate_limit)
            .unwrap_or(&tier_config.base_rate_limit);

        // Get or create tenant usage map
        let tenant_usage = usage
            .entry(tenant_id.to_string())
            .or_insert_with(HashMap::new);

        // Create endpoint key
        let endpoint_key = format!("{}:{}", method, path);
        
        let entry = tenant_usage
            .entry(endpoint_key.clone())
            .or_insert_with(|| (now, 0));

        let window = Duration::from_secs(limit_info.window_seconds as u64);
        if now.duration_since(entry.0) > window {
            // Reset window
            entry.0 = now;
            entry.1 = 0;
        }

        let remaining = limit_info.requests_per_window.saturating_sub(entry.1);
        
        if entry.1 >= limit_info.requests_per_window {
            let reset_time = entry.0 + window;
            
            // Record rate limit hit
            RATE_LIMIT_HITS.with_label_values(&[
                path,
                method.as_str(),
                tier,
            ]).inc();

            Err(handle_rate_limit_error(
                "Rate limit exceeded",
                Some(json!({
                    "tenant_id": tenant_id,
                    "tier": tier,
                    "endpoint": endpoint_key,
                    "limit": limit_info.requests_per_window,
                    "reset": reset_time.duration_since(now).as_secs(),
                }))
            ))
        } else {
            entry.1 += 1;
            Ok(RateLimitStatus::Allowed { remaining })
        }
    }
}

#[derive(Debug)]
enum RateLimitStatus {
    Allowed { remaining: u32 },
}

pub struct RateLimitMiddleware {
    config: RateLimitConfig,
}

impl<S> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            config: self.config.clone(),
        })
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    config: RateLimitConfig,
}

impl<S> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let tenant_id = req.headers()
            .get("X-Tenant-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("anonymous")
            .to_string();

        let tier = req.headers()
            .get("X-Subscription-Tier")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("free")
            .to_string();

        let path = req.path().to_string();
        let method = req.method().clone();

        let config = self.config.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            match config.check_rate_limit(&tenant_id, &tier, &path, &method).await {
                Ok(RateLimitStatus::Allowed { remaining }) => {
                    let mut response = fut.await?;
                    
                    // Add rate limit headers
                    let headers = response.headers_mut();
                    headers.insert(
                        "X-RateLimit-Remaining",
                        remaining.to_string().parse().unwrap()
                    );
                    
                    Ok(response)
                }
                Err(e) => {
                    warn!(
                        tenant_id = %tenant_id,
                        tier = %tier,
                        path = %path,
                        method = %method,
                        error = %e,
                        "Rate limit exceeded"
                    );
                    Err(e.into())
                }
            }
        })
    }
} 