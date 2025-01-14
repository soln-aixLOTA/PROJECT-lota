use opentelemetry::{
    metrics::{Counter, Histogram, Meter, MeterProvider, Unit},
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::sync::Arc;
use tokio::sync::OnceCell;

static METRICS: OnceCell<Arc<ApiMetrics>> = OnceCell::const_new();

/// API metrics for monitoring security-related events
pub struct ApiMetrics {
    request_counter: Counter<u64>,
    error_counter: Counter<u64>,
    auth_failures: Counter<u64>,
    request_duration: Histogram<f64>,
    active_connections: Counter<i64>,
}

impl ApiMetrics {
    pub fn global() -> Arc<ApiMetrics> {
        METRICS.get().expect("Metrics not initialized").clone()
    }

    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let exporter = opentelemetry_prometheus::exporter().init();
        let meter = exporter.meter_provider().meter("lota_ai_security_metrics");

        let metrics = Arc::new(Self::new(meter));
        METRICS.set(metrics).expect("Failed to set metrics");
        Ok(())
    }

    fn new(meter: Meter) -> Self {
        Self {
            request_counter: meter
                .u64_counter("api.requests.total")
                .with_description("Total number of API requests")
                .with_unit(Unit::new("requests"))
                .init(),

            error_counter: meter
                .u64_counter("api.errors.total")
                .with_description("Total number of API errors")
                .with_unit(Unit::new("errors"))
                .init(),

            auth_failures: meter
                .u64_counter("api.auth.failures")
                .with_description("Total number of authentication failures")
                .with_unit(Unit::new("failures"))
                .init(),

            request_duration: meter
                .f64_histogram("api.request.duration")
                .with_description("Request duration in seconds")
                .with_unit(Unit::new("seconds"))
                .init(),

            active_connections: meter
                .i64_counter("api.connections.active")
                .with_description("Number of active connections")
                .with_unit(Unit::new("connections"))
                .init(),
        }
    }

    pub fn record_request(&self, method: &str, path: &str) {
        self.request_counter.add(
            1,
            &[
                KeyValue::new("method", method.to_string()),
                KeyValue::new("path", path.to_string()),
            ],
        );
    }

    pub fn record_error(&self, error_type: &str, status_code: u16) {
        self.error_counter.add(
            1,
            &[
                KeyValue::new("error_type", error_type.to_string()),
                KeyValue::new("status_code", status_code.to_string()),
            ],
        );
    }

    pub fn record_auth_failure(&self, reason: &str) {
        self.auth_failures
            .add(1, &[KeyValue::new("reason", reason.to_string())]);
    }

    pub fn record_request_duration(&self, duration_secs: f64, path: &str) {
        self.request_duration
            .record(duration_secs, &[KeyValue::new("path", path.to_string())]);
    }

    pub fn connection_change(&self, delta: i64) {
        self.active_connections.add(delta, &[]);
    }
}

/// Get metrics in Prometheus format
pub fn get_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        ApiMetrics::init().expect("Failed to initialize metrics");
        let metrics = ApiMetrics::global();

        // Record some test metrics
        metrics.record_request("GET", "/api/test");
        metrics.record_error("validation_error", 400);
        metrics.record_auth_failure("invalid_token");
        metrics.record_request_duration(0.5, "/api/test");
        metrics.connection_change(1);

        // Get metrics output
        let output = get_metrics();
        assert!(output.contains("api_requests_total"));
        assert!(output.contains("api_errors_total"));
        assert!(output.contains("api_auth_failures"));
        assert!(output.contains("api_request_duration"));
        assert!(output.contains("api_connections_active"));
    }
}
