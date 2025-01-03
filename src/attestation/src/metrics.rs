use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, Histogram, HistogramVec, IntCounter,
    IntCounterVec,
};

lazy_static! {
    pub static ref ATTESTATION_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "attestation_requests_total",
        "Total number of attestation requests",
        &["status"]
    )
    .unwrap();
    pub static ref ATTESTATION_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "attestation_request_duration_seconds",
        "Attestation request duration in seconds",
        &["status"],
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap();
    pub static ref ATTESTATION_ERRORS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "attestation_errors_total",
        "Total number of attestation errors",
        &["type"]
    )
    .unwrap();
}

/// Record a successful attestation request
pub fn record_success(duration: f64) {
    ATTESTATION_REQUESTS_TOTAL
        .with_label_values(&["success"])
        .inc();
    ATTESTATION_REQUEST_DURATION
        .with_label_values(&["success"])
        .observe(duration);
}

/// Record a failed attestation request
pub fn record_failure(error_type: &str, duration: f64) {
    ATTESTATION_REQUESTS_TOTAL
        .with_label_values(&["failure"])
        .inc();
    ATTESTATION_REQUEST_DURATION
        .with_label_values(&["failure"])
        .observe(duration);
    ATTESTATION_ERRORS_TOTAL
        .with_label_values(&[error_type])
        .inc();
}

/// Initialize metrics endpoint
pub fn metrics_handler() -> impl actix_web::Responder {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&prometheus::gather(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
