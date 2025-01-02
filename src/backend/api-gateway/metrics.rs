use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};

lazy_static! {
    pub static ref REQUEST_COUNTER: IntCounterVec = register_int_counter_vec!(
        "api_gateway_requests_total",
        "Total number of requests processed",
        &["endpoint"]
    )
    .unwrap();
    pub static ref REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "api_gateway_request_duration_seconds",
        "Request duration in seconds",
        &["endpoint"],
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    )
    .unwrap();
    pub static ref ERROR_COUNTER: IntCounterVec = register_int_counter_vec!(
        "api_gateway_errors_total",
        "Total number of errors",
        &["error_type"]
    )
    .unwrap();
}

pub fn init_metrics() {
    // Initialize default metrics
    prometheus::default_registry();
}
