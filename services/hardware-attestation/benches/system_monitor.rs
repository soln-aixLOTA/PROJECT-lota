use std::future::Future;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use sysinfo::{System, SystemExt, CpuExt};
use hardware_attestation::NvmlWrapper;

/// Stores system metrics collected during benchmark execution
#[derive(Default)]
pub struct SystemMetrics {
    pub cpu_usage: Vec<f64>,
    pub memory_usage: Vec<f64>,
    pub gpu_memory_usage: Vec<f64>,
}

/// Monitors system metrics during benchmark execution
pub struct MonitoredBenchmark {
    metrics: Arc<Mutex<SystemMetrics>>,
    monitor_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl MonitoredBenchmark {
    pub fn new() -> Self {
        let metrics = Arc::new(Mutex::new(SystemMetrics::default()));
        let metrics_clone = Arc::clone(&metrics);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let monitor_thread = Some(thread::spawn(move || {
            let mut sys = System::new_all();
            let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");

            while running_clone.load(Ordering::SeqCst) {
                sys.refresh_all();

                // Get CPU usage (average across all cores)
                let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;

                // Get memory usage percentage
                let memory_usage = sys.used_memory() as f64 / sys.total_memory() as f64 * 100.0;

                // Get GPU memory usage percentage
                let gpu_memory_usage = nvml.get_device_memory_info(0)
                    .map(|info| (info.used as f64 / info.total as f64) * 100.0)
                    .unwrap_or(0.0);

                let mut metrics = metrics_clone.lock().unwrap();
                metrics.cpu_usage.push(cpu_usage as f64);
                metrics.memory_usage.push(memory_usage);
                metrics.gpu_memory_usage.push(gpu_memory_usage);
                drop(metrics); // Explicitly drop the lock

                thread::sleep(Duration::from_millis(100));
            }
        }));

        Self {
            metrics,
            monitor_thread,
            running,
        }
    }

    pub async fn run<F, Fut>(&self, iters: u64, mut op: F) -> Duration
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = ()>,
    {
        let start = std::time::Instant::now();
        for _ in 0..iters {
            op().await;
            // Log metrics after each iteration
            if let Ok(metrics) = self.metrics.lock() {
                if !metrics.cpu_usage.is_empty() {
                    let last_cpu = metrics.cpu_usage.last().unwrap();
                    let last_mem = metrics.memory_usage.last().unwrap();
                    let last_gpu = metrics.gpu_memory_usage.last().unwrap();
                    println!(
                        "Metrics - CPU: {:.2}%, System Memory: {:.2}%, GPU Memory: {:.2}%",
                        last_cpu, last_mem, last_gpu
                    );
                }
            }
        }
        start.elapsed()
    }
}

impl Drop for MonitoredBenchmark {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.monitor_thread.take() {
            let _ = thread.join();
        }
    }
}
