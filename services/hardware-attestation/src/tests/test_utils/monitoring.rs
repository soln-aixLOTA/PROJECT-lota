use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Resource usage metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    pub timestamp: Instant,
    pub cpu_usage: f32,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
}

/// Resource monitor for tests
pub struct ResourceMonitor {
    system: Arc<Mutex<System>>,
    metrics: Arc<Mutex<Vec<ResourceMetrics>>>,
    sample_interval: Duration,
}

impl ResourceMonitor {
    pub fn new(sample_interval: Duration) -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new_all())),
            metrics: Arc::new(Mutex::new(Vec::new())),
            sample_interval,
        }
    }

    /// Start monitoring resources
    pub async fn start_monitoring(&self) {
        let system = self.system.clone();
        let metrics = self.metrics.clone();
        let interval = self.sample_interval;

        tokio::spawn(async move {
            loop {
                let mut sys = system.lock().await;
                sys.refresh_all();

                let metric = ResourceMetrics {
                    timestamp: Instant::now(),
                    cpu_usage: sys.global_cpu_info().cpu_usage(),
                    memory_total: sys.total_memory(),
                    memory_used: sys.used_memory(),
                    memory_usage: sys.used_memory() as f64 / sys.total_memory() as f64,
                };

                metrics.lock().await.push(metric);

                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Get current metrics
    pub async fn get_current_metrics(&self) -> ResourceMetrics {
        let mut sys = self.system.lock().await;
        sys.refresh_all();

        ResourceMetrics {
            timestamp: Instant::now(),
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            memory_total: sys.total_memory(),
            memory_used: sys.used_memory(),
            memory_usage: sys.used_memory() as f64 / sys.total_memory() as f64,
        }
    }

    /// Get metrics history
    pub async fn get_metrics_history(&self) -> Vec<ResourceMetrics> {
        self.metrics.lock().await.clone()
    }

    /// Check if resource usage exceeds thresholds
    pub async fn check_thresholds(&self, cpu_threshold: f32, memory_threshold: f64) -> bool {
        let metrics = self.get_current_metrics().await;

        if metrics.cpu_usage > cpu_threshold {
            warn!("CPU usage exceeds threshold: {:.2}%", metrics.cpu_usage);
            return false;
        }

        if metrics.memory_usage > memory_threshold {
            warn!(
                "Memory usage exceeds threshold: {:.2}%",
                metrics.memory_usage * 100.0
            );
            return false;
        }

        true
    }
}

/// RAII guard for resource monitoring
pub struct MonitoringGuard {
    monitor: Arc<ResourceMonitor>,
}

impl MonitoringGuard {
    pub fn new(sample_interval: Duration) -> Self {
        let monitor = Arc::new(ResourceMonitor::new(sample_interval));
        let guard = Self {
            monitor: monitor.clone(),
        };

        tokio::spawn(async move {
            monitor.start_monitoring().await;
        });

        guard
    }

    pub async fn get_metrics(&self) -> Vec<ResourceMetrics> {
        self.monitor.get_metrics_history().await
    }

    pub async fn check_thresholds(&self, cpu_threshold: f32, memory_threshold: f64) -> bool {
        self.monitor
            .check_thresholds(cpu_threshold, memory_threshold)
            .await
    }
}
