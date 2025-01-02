use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorTooManyRequests;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, Result};
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct RateLimiter {
    redis_pool: deadpool_redis::Pool,
    requests: u32,
    duration: u32,
}

impl RateLimiter {
    pub fn new(redis_pool: deadpool_redis::Pool, requests: u32, duration: u32) -> Self {
        Self {
            redis_pool,
            requests,
            duration,
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
            redis_pool: self.redis_pool.clone(),
            requests: self.requests,
            duration: self.duration,
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    redis_pool: deadpool_redis::Pool,
    requests: u32,
    duration: u32,
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
        let redis_pool = self.redis_pool.clone();
        let requests = self.requests;
        let duration = self.duration;
        let fut = self.service.call(req);

        Box::pin(async move {
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("unknown")
                .to_string();
            let path = req.path().to_string();
            let key = format!("rate_limit:{}:{}", ip, path);

            let mut conn = redis_pool.get().await.map_err(|e| {
                error!("Redis connection error: {}", e);
                ErrorTooManyRequests("Rate limit error")
            })?;

            // Get current count
            let count: Option<u32> = conn.get(&key).await.map_err(|e| {
                error!("Redis get error: {}", e);
                ErrorTooManyRequests("Rate limit error")
            })?;

            match count {
                Some(c) if c >= requests => {
                    // Get TTL
                    let ttl: Option<u32> = conn.ttl(&key).await.map_err(|e| {
                        error!("Redis TTL error: {}", e);
                        ErrorTooManyRequests("Rate limit error")
                    })?;

                    warn!("Rate limit exceeded for IP: {}, Path: {}", ip, path);
                    let mut response = actix_web::HttpResponse::TooManyRequests();
                    if let Some(reset) = ttl {
                        response.insert_header(("X-RateLimit-Reset", reset.to_string()));
                    }
                    response.insert_header(("X-RateLimit-Limit", requests.to_string()));
                    response.insert_header(("X-RateLimit-Remaining", "0"));
                    return Err(ErrorTooManyRequests(response.finish()));
                }
                Some(c) => {
                    // Increment
                    let new_count: u32 = conn.incr(&key, 1).await.map_err(|e| {
                        error!("Redis increment error: {}", e);
                        ErrorTooManyRequests("Rate limit error")
                    })?;

                    let mut response = fut.await?;
                    response.headers_mut().insert(
                        HeaderName::from_static("x-ratelimit-limit"),
                        HeaderValue::from_str(&requests.to_string()).unwrap(),
                    );
                    response.headers_mut().insert(
                        HeaderName::from_static("x-ratelimit-remaining"),
                        HeaderValue::from_str(&(requests - new_count).to_string()).unwrap(),
                    );
                    Ok(response)
                }
                None => {
                    // First request
                    conn.set_ex(&key, 1, duration as usize).await.map_err(|e| {
                        error!("Redis set error: {}", e);
                        ErrorTooManyRequests("Rate limit error")
                    })?;

                    let mut response = fut.await?;
                    response.headers_mut().insert(
                        HeaderName::from_static("x-ratelimit-limit"),
                        HeaderValue::from_str(&requests.to_string()).unwrap(),
                    );
                    response.headers_mut().insert(
                        HeaderName::from_static("x-ratelimit-remaining"),
                        HeaderValue::from_str(&(requests - 1).to_string()).unwrap(),
                    );
                    Ok(response)
                }
            }
        })
    }
}
