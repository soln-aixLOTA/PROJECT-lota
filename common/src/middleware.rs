use std::future::{ready, Ready};
use std::sync::Arc;
use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid tenant")]
    InvalidTenant,
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn process(&self, ctx: &mut Context) -> Result<(), MiddlewareError>;
}

pub struct Context {
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub request_id: String,
    pub path: String,
    pub method: String,
}

impl Context {
    pub fn new(path: String, method: String) -> Self {
        Self {
            tenant_id: None,
            user_id: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            path,
            method,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

pub struct RateLimitMiddleware {
    config: Arc<RateLimitConfig>,
}

impl RateLimitMiddleware {
    pub fn new(config: Arc<RateLimitConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Middleware for RateLimitMiddleware {
    async fn process(&self, ctx: &mut Context) -> Result<(), MiddlewareError> {
        // Implement rate limiting logic here
        // This is a placeholder implementation
        info!(
            request_id = %ctx.request_id,
            path = %ctx.path,
            "Rate limit check passed"
        );
        Ok(())
    }
}

pub struct LoggingMiddleware {
    log_level: tracing::Level,
}

impl LoggingMiddleware {
    pub fn new(log_level: tracing::Level) -> Self {
        Self { log_level }
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn process(&self, ctx: &mut Context) -> Result<(), MiddlewareError> {
        info!(
            request_id = %ctx.request_id,
            path = %ctx.path,
            method = %ctx.method,
            tenant_id = ?ctx.tenant_id,
            user_id = ?ctx.user_id,
            "Processing request"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_middleware() {
        let config = Arc::new(RateLimitConfig {
            requests_per_second: 10,
            burst_size: 5,
        });
        let middleware = RateLimitMiddleware::new(config);
        let mut ctx = Context::new("/test".to_string(), "GET".to_string());

        assert!(middleware.process(&mut ctx).await.is_ok());
    }

    #[tokio::test]
    async fn test_logging_middleware() {
        let middleware = LoggingMiddleware::new(tracing::Level::INFO);
        let mut ctx = Context::new("/test".to_string(), "POST".to_string());
        ctx.tenant_id = Some("test-tenant".to_string());
        ctx.user_id = Some("test-user".to_string());

        assert!(middleware.process(&mut ctx).await.is_ok());
    }
} 