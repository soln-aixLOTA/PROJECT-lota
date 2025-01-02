use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, Histogram, HistogramVec,
    IntCounter, IntCounterVec, IntGauge,
};
use std::time::Instant;

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["method", "path", "status"]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();
    pub static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "active_connections",
        "Number of currently active connections"
    )
    .unwrap();
    pub static ref DATABASE_CONNECTIONS: IntGauge = register_int_gauge!(
        "database_connections",
        "Number of active database connections"
    )
    .unwrap();
    pub static ref REDIS_CONNECTIONS: IntGauge =
        register_int_gauge!("redis_connections", "Number of active Redis connections").unwrap();
    pub static ref RATE_LIMIT_HITS: IntCounterVec = register_int_counter_vec!(
        "rate_limit_hits_total",
        "Total number of rate limit hits",
        &["path"]
    )
    .unwrap();
    pub static ref AUTH_FAILURES: IntCounterVec = register_int_counter_vec!(
        "auth_failures_total",
        "Total number of authentication failures",
        &["reason"]
    )
    .unwrap();
    pub static ref TOKEN_VALIDATIONS: IntCounterVec = register_int_counter_vec!(
        "token_validations_total",
        "Total number of token validations",
        &["status"]
    )
    .unwrap();
    pub static ref API_ERRORS: IntCounterVec = register_int_counter_vec!(
        "api_errors_total",
        "Total number of API errors",
        &["type", "code"]
    )
    .unwrap();
}

pub struct RequestMetrics {
    pub path: String,
    pub method: String,
    pub start_time: Instant,
}

impl RequestMetrics {
    pub fn new(path: String, method: String) -> Self {
        ACTIVE_CONNECTIONS.inc();
        Self {
            path,
            method,
            start_time: Instant::now(),
        }
    }

    pub fn record_response(&self, status: u16) {
        let duration = self.start_time.elapsed().as_secs_f64();
        HTTP_REQUESTS_TOTAL
            .with_label_values(&[&self.method, &self.path, &status.to_string()])
            .inc();
        HTTP_REQUEST_DURATION
            .with_label_values(&[&self.method, &self.path])
            .observe(duration);
        ACTIVE_CONNECTIONS.dec();
    }
}

pub fn record_rate_limit_hit(path: &str) {
    RATE_LIMIT_HITS.with_label_values(&[path]).inc();
}

pub fn record_auth_failure(reason: &str) {
    AUTH_FAILURES.with_label_values(&[reason]).inc();
}

pub fn record_token_validation(status: &str) {
    TOKEN_VALIDATIONS.with_label_values(&[status]).inc();
}

pub fn record_api_error(error_type: &str, code: &str) {
    API_ERRORS.with_label_values(&[error_type, code]).inc();
}

pub fn update_database_connections(count: i64) {
    DATABASE_CONNECTIONS.set(count);
}

pub fn update_redis_connections(count: i64) {
    REDIS_CONNECTIONS.set(count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_request_metrics() {
        let metrics = RequestMetrics::new("/test".to_string(), "GET".to_string());
        thread::sleep(Duration::from_millis(100));
        metrics.record_response(200);

        let counter = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/test", "200"])
            .get();
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_rate_limit_metrics() {
        record_rate_limit_hit("/api/test");
        let counter = RATE_LIMIT_HITS.with_label_values(&["/api/test"]).get();
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_auth_metrics() {
        record_auth_failure("invalid_token");
        let counter = AUTH_FAILURES.with_label_values(&["invalid_token"]).get();
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_connection_metrics() {
        update_database_connections(5);
        assert_eq!(DATABASE_CONNECTIONS.get(), 5);

        update_redis_connections(3);
        assert_eq!(REDIS_CONNECTIONS.get(), 3);
    }
}
