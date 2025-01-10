use anyhow::Result;
use serde::Serialize;
use std::str::FromStr;
use std::sync::Once;
use tracing::{info, Level};
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

#[derive(Debug, Serialize)]
pub struct LogContext {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub service: String,
    pub environment: String,
}

impl LogContext {
    pub fn new(service: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            tenant_id: None,
            user_id: None,
            service: service.into(),
            environment: environment.into(),
        }
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

pub fn init_logging(log_level: &str) -> anyhow::Result<()> {
    let env_filter = EnvFilter::from_str(log_level).unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "{}={}",
            env!("CARGO_PKG_NAME"),
            log_level.to_lowercase()
        ))
    });

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .init();

    tracing::info!("Logging initialized at level: {}", log_level);
    Ok(())
}

pub fn log_request(ctx: &LogContext, message: &str) {
    info!(
        request_id = %ctx.request_id,
        tenant_id = ?ctx.tenant_id,
        user_id = ?ctx.user_id,
        service = %ctx.service,
        environment = %ctx.environment,
        "{}", message
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        let ctx = LogContext::new("api-gateway", "development")
            .with_tenant("test-tenant")
            .with_user("test-user");

        assert_eq!(ctx.service, "api-gateway");
        assert_eq!(ctx.environment, "development");
        assert_eq!(ctx.tenant_id, Some("test-tenant".to_string()));
        assert_eq!(ctx.user_id, Some("test-user".to_string()));
    }

    #[test]
    fn test_logging_initialization() {
        assert!(init_logging("info").is_ok());
        // Second initialization should also be ok due to Once guard
        assert!(init_logging("info").is_ok());
    }
}
