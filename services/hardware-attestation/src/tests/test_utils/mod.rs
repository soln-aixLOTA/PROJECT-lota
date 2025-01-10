mod cleanup;
mod error_injection;
mod monitoring;

pub use cleanup::{CleanupGuard, TestCleanup};
pub use error_injection::{
    scenarios as error_scenarios, ErrorConfig, ErrorInjectionLayer, ErrorInjector,
};
pub use monitoring::{MonitoringGuard, ResourceMetrics, ResourceMonitor};

use std::time::Duration;

/// Default test configuration
pub const DEFAULT_SAMPLE_INTERVAL: Duration = Duration::from_millis(100);
pub const DEFAULT_CPU_THRESHOLD: f32 = 80.0;
pub const DEFAULT_MEMORY_THRESHOLD: f64 = 0.8;

/// Test setup helper that provides cleanup and monitoring
pub struct TestContext {
    pub cleanup: CleanupGuard,
    pub monitoring: MonitoringGuard,
}

impl TestContext {
    pub fn new(cleanup: TestCleanup) -> Self {
        Self {
            cleanup: CleanupGuard::new(cleanup),
            monitoring: MonitoringGuard::new(DEFAULT_SAMPLE_INTERVAL),
        }
    }

    pub async fn check_resource_usage(&self) -> bool {
        self.monitoring
            .check_thresholds(DEFAULT_CPU_THRESHOLD, DEFAULT_MEMORY_THRESHOLD)
            .await
    }

    pub async fn get_metrics(&self) -> Vec<ResourceMetrics> {
        self.monitoring.get_metrics().await
    }
}

/// Helper function to create a test context with error injection
pub fn setup_test_context_with_errors<S>(
    service: S,
    cleanup: TestCleanup,
    error_config: ErrorConfig,
) -> (TestContext, ErrorInjector<S>) {
    let context = TestContext::new(cleanup);
    let error_injector = ErrorInjector::new(service, error_config);
    (context, error_injector)
}
