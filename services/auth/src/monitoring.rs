use metrics::{counter, histogram};
use std::time::Instant;

pub struct SecretMetrics;

impl SecretMetrics {
    pub fn record_secret_retrieval(secret_name: &str, success: bool) {
        if success {
            counter!("secret_retrieval_success_count", 1, "secret_name" => secret_name.to_string());
        } else {
            counter!("secret_retrieval_failure_count", 1, "secret_name" => secret_name.to_string());
        }
    }

    pub fn record_secret_rotation(secret_name: &str, success: bool) {
        if success {
            counter!("secret_rotation_success_count", 1, "secret_name" => secret_name.to_string());
        } else {
            counter!("secret_rotation_failure_count", 1, "secret_name" => secret_name.to_string());
        }
    }

    pub fn record_secret_operation_duration(operation: &str, duration: std::time::Duration) {
        histogram!("secret_operation_duration_seconds", duration.as_secs_f64(), "operation" => operation.to_string());
    }
}

pub struct SecretOperationTimer {
    operation: String,
    start_time: Instant,
}

impl SecretOperationTimer {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            start_time: Instant::now(),
        }
    }
}

impl Drop for SecretOperationTimer {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        SecretMetrics::record_secret_operation_duration(&self.operation, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_secret_metrics_recording() {
        // Record successful secret retrieval
        SecretMetrics::record_secret_retrieval("test_secret", true);

        // Record failed secret retrieval
        SecretMetrics::record_secret_retrieval("test_secret", false);

        // Record successful secret rotation
        SecretMetrics::record_secret_rotation("test_secret", true);

        // Record failed secret rotation
        SecretMetrics::record_secret_rotation("test_secret", false);

        // Record operation duration
        SecretMetrics::record_secret_operation_duration("get_secret", Duration::from_secs(1));
    }

    #[test]
    fn test_secret_operation_timer() {
        let _timer = SecretOperationTimer::new("test_operation");
        std::thread::sleep(Duration::from_millis(100));
        // Timer will automatically record duration when dropped
    }
}
