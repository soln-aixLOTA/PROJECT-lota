use serde::Serialize;
use std::str::FromStr;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
};
use uuid;

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

/// Initialize logging with the specified log level
pub fn init_logging(level: Option<String>) -> anyhow::Result<()> {
    let env_filter = match level {
        Some(level) => {
            let level = Level::from_str(&level).unwrap_or(Level::INFO);
            EnvFilter::new(format!("{}={}", module_path!(), level))
        }
        None => EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
    };

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    Ok(())
}

pub fn log_request(ctx: &LogContext, message: &str) {
    tracing::info!(
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
    fn test_init_logging() {
        assert!(init_logging(None).is_ok());
        assert!(init_logging(Some("debug".to_string())).is_ok());
        assert!(init_logging(Some("invalid".to_string())).is_ok()); // Falls back to INFO
    }
}
