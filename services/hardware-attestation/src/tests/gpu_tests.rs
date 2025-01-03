#[cfg(test)]
mod tests {
    use crate::core::gpu_monitor::{GPUMonitor, HealthStatus};
    use anyhow::Result;
    use test_log::test;
    use tokio::time::Duration;

    #[test(tokio::test)]
    async fn test_gpu_monitor_initialization() -> Result<()> {
        let monitor = GPUMonitor::new(Duration::from_secs(1))?;
        monitor.start_monitoring().await?;

        // Wait for initial metrics collection
        tokio::time::sleep(Duration::from_secs(2)).await;

        let metrics = monitor.get_gpu_metrics().await;
        assert!(!metrics.is_empty(), "No GPU metrics found");

        for (gpu_id, gpu_metrics) in metrics {
            println!("GPU {}: {:?}", gpu_id, gpu_metrics);
            assert!(!gpu_id.is_empty(), "GPU ID should not be empty");
            assert!(
                gpu_metrics.memory_total > 0,
                "Total memory should be greater than 0"
            );
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_gpu_process_monitoring() -> Result<()> {
        let monitor = GPUMonitor::new(Duration::from_secs(1))?;
        monitor.start_monitoring().await?;

        let processes = monitor.get_gpu_processes().await?;
        assert!(!processes.is_empty(), "No GPU processes found");

        for (gpu_id, gpu_processes) in processes {
            println!("GPU {}: {} processes", gpu_id, gpu_processes.len());
            for process in gpu_processes {
                assert!(process.pid > 0, "Process ID should be greater than 0");
                println!(
                    "  PID {}: {} MB",
                    process.pid,
                    process.memory_used / 1024 / 1024
                );
            }
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_gpu_health_checks() -> Result<()> {
        let monitor = GPUMonitor::new(Duration::from_secs(1))?;
        monitor.start_monitoring().await?;

        let health_checks = monitor.check_gpu_health().await?;
        assert!(!health_checks.is_empty(), "No health checks found");

        for (gpu_id, checks) in health_checks {
            println!("GPU {}: {} health checks", gpu_id, checks.len());
            for check in checks {
                println!(
                    "  {:?}: {:?} - {}",
                    check.check_type, check.status, check.message
                );
                match check.status {
                    HealthStatus::OK => {
                        assert_eq!(check.message, "All metrics within normal ranges");
                    }
                    HealthStatus::Warning => {
                        assert!(
                            !check.message.is_empty(),
                            "Warning message should not be empty"
                        );
                    }
                    HealthStatus::Critical => {
                        assert!(
                            !check.message.is_empty(),
                            "Critical message should not be empty"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_gpu_metrics_update() -> Result<()> {
        let monitor = GPUMonitor::new(Duration::from_secs(1))?;
        monitor.start_monitoring().await?;

        // Get initial metrics
        let initial_metrics = monitor.get_gpu_metrics().await;

        // Wait for metrics update
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Get updated metrics
        let updated_metrics = monitor.get_gpu_metrics().await;

        assert_eq!(
            initial_metrics.len(),
            updated_metrics.len(),
            "Number of GPUs should remain constant"
        );

        for (gpu_id, metrics) in &updated_metrics {
            let initial = initial_metrics.get(gpu_id).unwrap();
            assert!(
                metrics.timestamp > initial.timestamp,
                "Metrics should be updated"
            );
        }

        Ok(())
    }
}
