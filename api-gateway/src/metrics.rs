use lazy_static::lazy_static;
use prometheus::{
    register_histogram, register_int_counter, register_int_gauge,
    Histogram, IntCounter, IntGauge,
};
use std::sync::Arc;

lazy_static! {
    // System-wide metrics
    pub static ref ACTIVE_REQUESTS: IntGauge = register_int_gauge!(
        "api_gateway_active_requests",
        "Number of currently active requests"
    )
    .unwrap();

    pub static ref REQUEST_DURATION: Histogram = register_histogram!(
        "api_gateway_request_duration_seconds",
        "Request duration in seconds"
    )
    .unwrap();

    pub static ref SYSTEM_LOAD: IntGauge = register_int_gauge!(
        "api_gateway_system_load",
        "Current system load average"
    )
    .unwrap();

    pub static ref GPU_UTILIZATION: IntGauge = register_int_gauge!(
        "api_gateway_gpu_utilization",
        "Current GPU utilization percentage"
    )
    .unwrap();

    pub static ref ERROR_COUNT: IntCounter = register_int_counter!(
        "api_gateway_errors_total",
        "Total number of errors"
    )
    .unwrap();

    pub static ref REQUEST_COUNT: IntCounter = register_int_counter!(
        "api_gateway_requests_total",
        "Total number of requests"
    )
    .unwrap();
}

#[derive(Clone)]
pub struct WorkerMetrics {
    pub requests_processed: Arc<IntCounter>,
    pub processing_time: Arc<Histogram>,
    pub error_count: Arc<IntCounter>,
    pub queue_size: Arc<IntGauge>,
}

impl WorkerMetrics {
    pub fn new(worker_id: usize) -> Self {
        Self {
            requests_processed: Arc::new(
                register_int_counter!(
                    "api_gateway_worker_requests_total",
                    "Total requests processed by worker",
                    &["worker_id"]
                )
                .unwrap(),
            ),
            processing_time: Arc::new(
                register_histogram!(
                    "api_gateway_worker_processing_time_seconds",
                    "Request processing time in seconds",
                    &["worker_id"]
                )
                .unwrap(),
            ),
            error_count: Arc::new(
                register_int_counter!(
                    "api_gateway_worker_errors_total",
                    "Total errors encountered by worker",
                    &["worker_id"]
                )
                .unwrap(),
            ),
            queue_size: Arc::new(
                register_int_gauge!(
                    "api_gateway_worker_queue_size",
                    "Current size of worker's request queue",
                    &["worker_id"]
                )
                .unwrap(),
            ),
        }
    }
}

impl Default for WorkerMetrics {
    fn default() -> Self {
        Self::new(0)
    }
}

pub fn init_metrics() {
    // Initialize any additional metrics setup here
    info!("Metrics system initialized");
}

// Helper functions for common metric operations
pub fn record_request_start() {
    ACTIVE_REQUESTS.inc();
    REQUEST_COUNT.inc();
}

pub fn record_request_end(duration_seconds: f64) {
    ACTIVE_REQUESTS.dec();
    REQUEST_DURATION.observe(duration_seconds);
}

pub fn record_error() {
    ERROR_COUNT.inc();
}

pub fn update_system_metrics(cpu_load: f64, gpu_util: Option<f64>) {
    SYSTEM_LOAD.set(cpu_load as i64);
    if let Some(gpu) = gpu_util {
        GPU_UTILIZATION.set(gpu as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_metrics_creation() {
        let metrics = WorkerMetrics::new(1);
        assert!(Arc::strong_count(&metrics.requests_processed) == 1);
        assert!(Arc::strong_count(&metrics.processing_time) == 1);
        assert!(Arc::strong_count(&metrics.error_count) == 1);
        assert!(Arc::strong_count(&metrics.queue_size) == 1);
    }

    #[test]
    fn test_metric_recording() {
        record_request_start();
        assert_eq!(ACTIVE_REQUESTS.get(), 1);
        
        record_request_end(0.1);
        assert_eq!(ACTIVE_REQUESTS.get(), 0);
        
        record_error();
        assert_eq!(ERROR_COUNT.get(), 1);
    }
} 