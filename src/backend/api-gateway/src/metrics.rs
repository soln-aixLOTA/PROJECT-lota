use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge, Histogram, HistogramVec,
    IntCounter, IntCounterVec, IntGauge,
};

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
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap();
    pub static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "active_connections",
        "Number of currently active connections"
    )
    .unwrap();
    pub static ref RATE_LIMIT_HITS: IntCounterVec = register_int_counter_vec!(
        "rate_limit_hits_total",
        "Total number of rate limit hits",
        &["path"]
    )
    .unwrap();
    pub static ref UPSTREAM_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "upstream_request_duration_seconds",
        "Upstream service request duration in seconds",
        &["service"],
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap();
    pub static ref UPSTREAM_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "upstream_errors_total",
        "Total number of upstream service errors",
        &["service", "type"]
    )
    .unwrap();
}

/// Record an HTTP request
pub fn record_request(method: &str, path: &str, status: u16, duration: f64) {
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, path, &status.to_string()])
        .inc();
    HTTP_REQUEST_DURATION
        .with_label_values(&[method, path])
        .observe(duration);
}

/// Record a rate limit hit
pub fn record_rate_limit(path: &str) {
    RATE_LIMIT_HITS.with_label_values(&[path]).inc();
}

/// Record an upstream service request
pub fn record_upstream_request(service: &str, duration: f64, error_type: Option<&str>) {
    UPSTREAM_REQUEST_DURATION
        .with_label_values(&[service])
        .observe(duration);

    if let Some(err_type) = error_type {
        UPSTREAM_ERRORS_TOTAL
            .with_label_values(&[service, err_type])
            .inc();
    }
}

/// Update active connections count
pub fn update_connections(delta: i64) {
    ACTIVE_CONNECTIONS.add(delta);
}

/// Initialize metrics endpoint
pub fn metrics_handler() -> impl actix_web::Responder {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
