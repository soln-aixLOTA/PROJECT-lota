pub mod logging;
pub mod metrics;
pub mod tracing;

use thiserror::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("Logging error: {0}")]
    LoggingError(String),
    #[error("Metrics error: {0}")]
    MetricsError(String),
    #[error("Tracing error: {0}")]
    TracingError(String),
}

pub type Result<T> = std::result::Result<T, UtilError>;

/// Initialize logging with the given log level
pub fn init_logging(log_level: &str) -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .try_init()
        .map_err(|e| UtilError::LoggingError(e.to_string()))
}

/// Initialize metrics collection
pub fn init_metrics() -> Result<()> {
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([0, 0, 0, 0], 9000))
        .install()
        .map_err(|e| UtilError::MetricsError(e.to_string()))
}

/// Initialize distributed tracing
pub fn init_tracing(service_name: &str) -> Result<()> {
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(service_name)
        .install_simple()
        .map_err(|e| UtilError::TracingError(e.to_string()))?;
    Ok(())
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }
}
