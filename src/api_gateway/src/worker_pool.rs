use tokio::sync::{mpsc, RwLock, Semaphore};
use std::sync::Arc;
use std::time::{Duration, Instant};
use prometheus::{IntCounter, IntGauge, Histogram};
use tracing::{info, error};

use crate::error::ApiError;
use crate::metrics::WorkerMetrics;
use crate::config::AppConfig;

pub struct AdaptiveWorkerPool {
    semaphore: Arc<Semaphore>,
    workers: Vec<Worker>,
    metrics: Arc<WorkerMetrics>,
    config: Arc<AppConfig>,
}

impl AdaptiveWorkerPool {
    pub async fn new(config: Arc<AppConfig>) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        let metrics = Arc::new(WorkerMetrics::new());
        let mut workers = Vec::with_capacity(config.min_workers);

        for id in 0..config.min_workers {
            workers.push(Worker::new(
                id,
                Arc::clone(&semaphore),
                Arc::clone(&metrics),
                Arc::clone(&config),
            ));
        }

        Self {
            semaphore,
            workers,
            metrics,
            config,
        }
    }

    pub async fn process_request(&self, request: Request) -> Result<Response, ApiError> {
        let start = Instant::now();
        let permit = self.semaphore.acquire().await.map_err(|e| {
            ApiError::new(
                "CONCURRENCY_LIMIT_EXCEEDED",
                "Server is at maximum capacity",
                &request.id,
            )
        })?;

        self.metrics.active_requests.inc();

        let result = match self.select_worker().await {
            Some(worker) => worker.process(request).await,
            None => Err(ApiError::new(
                "NO_WORKERS_AVAILABLE",
                "No workers available to process request",
                &request.id,
            )),
        };

        self.metrics.active_requests.dec();
        self.metrics.request_duration.observe(start.elapsed().as_secs_f64());

        drop(permit);
        result
    }

    async fn select_worker(&self) -> Option<&Worker> {
        self.workers
            .iter()
            .min_by_key(|w| w.metrics.queue_size.get())
    }

    pub async fn monitor_health(&self) {
        let mut interval = tokio::time::interval(self.config.load_check_interval);

        loop {
            interval.tick().await;
            self.update_system_metrics().await;
            self.adjust_worker_count().await;
        }
    }

    async fn update_system_metrics(&self) {
        if let Ok(load) = sys_info::loadavg() {
            self.metrics.system_load.set(load.one as i64);
        }

        // Update GPU metrics if available
        #[cfg(feature = "gpu")]
        if let Ok(gpu_util) = self.get_gpu_utilization().await {
            self.metrics.gpu_utilization.set(gpu_util as i64);
        }
    }

    async fn adjust_worker_count(&self) {
        let current_load = sys_info::loadavg().map(|l| l.one).unwrap_or(0.0);
        let current_workers = self.workers.len();
        
        // Calculate comprehensive load metrics
        let queue_metrics = self.calculate_queue_metrics().await;
        let system_metrics = self.get_system_metrics().await;
        
        // Calculate target workers based on multiple factors
        let target_workers = self.calculate_target_workers(
            current_load,
            queue_metrics,
            system_metrics,
        ).await;

        info!(
            "System metrics - Load: {:.2}, CPU: {:.2}%, Memory: {:.2}%, Workers: {}, Avg Queue: {:.2}",
            current_load,
            system_metrics.cpu_usage,
            system_metrics.memory_usage,
            current_workers,
            queue_metrics.avg_queue_size,
        );

        // Apply scaling with hysteresis to prevent oscillation
        if target_workers > current_workers && 
           target_workers >= current_workers * (1.0 + self.config.scale_up_threshold) as usize {
            info!(
                "Scaling up workers from {} to {}",
                current_workers, target_workers
            );
            self.scale_up(target_workers - current_workers).await;
        } else if target_workers < current_workers && 
                  target_workers <= current_workers * (1.0 - self.config.scale_down_threshold) as usize {
            info!(
                "Scaling down workers from {} to {}",
                current_workers, target_workers
            );
            self.scale_down(current_workers - target_workers).await;
        }
    }

    async fn calculate_queue_metrics(&self) -> QueueMetrics {
        let queue_sizes: Vec<i64> = self.workers.iter()
            .map(|w| w.metrics.queue_size.get())
            .collect();
        
        let total_size: i64 = queue_sizes.iter().sum();
        let avg_size = if !queue_sizes.is_empty() {
            total_size as f64 / queue_sizes.len() as f64
        } else {
            0.0
        };
        
        // Calculate standard deviation for queue sizes
        let variance = if !queue_sizes.is_empty() {
            queue_sizes.iter()
                .map(|&size| {
                    let diff = size as f64 - avg_size;
                    diff * diff
                })
                .sum::<f64>() / queue_sizes.len() as f64
        } else {
            0.0
        };
        
        QueueMetrics {
            total_size: total_size as usize,
            avg_queue_size: avg_size,
            std_dev: variance.sqrt(),
            max_size: queue_sizes.iter().max().copied().unwrap_or(0) as usize,
        }
    }

    async fn get_system_metrics(&self) -> SystemMetrics {
        let mut metrics = SystemMetrics::default();
        
        // Get CPU usage
        if let Ok(cpu) = sys_info::cpu_usage() {
            metrics.cpu_usage = cpu;
        }
        
        // Get memory usage
        if let Ok(mem) = sys_info::mem_info() {
            metrics.memory_usage = (mem.total - mem.free) as f64 / mem.total as f64 * 100.0;
        }
        
        // Get GPU metrics if available
        #[cfg(feature = "gpu")]
        if let Ok(gpu) = self.get_gpu_utilization().await {
            metrics.gpu_usage = gpu;
        }
        
        metrics
    }

    async fn calculate_target_workers(
        &self,
        current_load: f64,
        queue_metrics: QueueMetrics,
        system_metrics: SystemMetrics,
    ) -> usize {
        // Calculate load factor based on multiple metrics
        let cpu_factor = (system_metrics.cpu_usage / 100.0).min(1.0);
        let memory_factor = (system_metrics.memory_usage / 100.0).min(1.0);
        let queue_factor = if queue_metrics.avg_queue_size > self.config.worker_queue_size as f64 * 0.8 {
            1.2 // High queue pressure
        } else if queue_metrics.avg_queue_size < self.config.worker_queue_size as f64 * 0.2 {
            0.8 // Low queue pressure
        } else {
            1.0
        };
        
        // Consider queue size variation
        let variation_factor = if queue_metrics.std_dev > queue_metrics.avg_queue_size * 0.5 {
            1.1 // High variation suggests need for more workers
        } else {
            1.0
        };
        
        // Calculate composite load factor
        let load_factor = f64::max(
            cpu_factor,
            f64::max(
                memory_factor,
                queue_factor * variation_factor
            ),
        );
        
        // Calculate base target
        let base_target = (self.config.min_workers as f64 * load_factor).ceil() as usize;
        
        // Apply bounds
        base_target.clamp(
            self.config.min_workers,
            self.config.max_workers,
        )
    }

    async fn scale_up(&self, count: usize) {
        let mut workers = self.workers.write().await;
        let current_count = workers.len();

        for id in current_count..(current_count + count) {
            workers.push(Worker::new(
                id,
                Arc::clone(&self.semaphore),
                Arc::clone(&self.metrics),
                Arc::clone(&self.config),
            ));
        }
    }

    async fn scale_down(&self, count: usize) {
        let mut workers = self.workers.write().await;
        let current_count = workers.len();
        let target_count = current_count.saturating_sub(count);

        // Remove workers with the highest queue sizes
        workers.sort_by_key(|w| std::cmp::Reverse(w.metrics.queue_size.get()));
        workers.truncate(target_count);
    }

    #[cfg(feature = "gpu")]
    async fn get_gpu_utilization(&self) -> Result<f64, Box<dyn std::error::Error>> {
        use nvml_wrapper::Nvml;
        
        let nvml = Nvml::init()?;
        let device = nvml.device_by_index(0)?;
        let utilization = device.utilization_rates()?;
        
        Ok(utilization.gpu as f64)
    }
}

struct Worker {
    id: usize,
    metrics: Arc<WorkerMetrics>,
    request_tx: mpsc::Sender<Request>,
    response_rx: mpsc::Receiver<Result<Response, ApiError>>,
}

impl Worker {
    fn new(
        id: usize,
        semaphore: Arc<Semaphore>,
        pool_metrics: Arc<WorkerMetrics>,
        config: Arc<AppConfig>,
    ) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel(config.worker_queue_size);
        let (response_tx, response_rx) = mpsc::channel(1);
        let metrics = Arc::new(WorkerMetrics::new(id));

        tokio::spawn(async move {
            while let Some(request) = request_rx.recv().await {
                let start = Instant::now();
                let result = Self::handle_request(request).await;
                
                metrics.processing_time.observe(start.elapsed().as_secs_f64());
                metrics.requests_processed.inc();

                if result.is_err() {
                    metrics.error_count.inc();
                }

                if response_tx.send(result).await.is_err() {
                    error!("Failed to send response for worker {}", id);
                }
            }
        });

        Self {
            id,
            metrics: Arc::new(metrics),
            request_tx,
            response_rx,
        }
    }

    async fn process(&self, request: Request) -> Result<Response, ApiError> {
        self.metrics.queue_size.inc();
        
        let result = self.request_tx.send(request).await.map_err(|e| {
            ApiError::new(
                "WORKER_QUEUE_FULL",
                "Worker queue is full",
                "system",
            )
        })?;

        self.metrics.queue_size.dec();
        self.response_rx.recv().await.unwrap_or_else(|| {
            Err(ApiError::new(
                "WORKER_ERROR",
                "Worker failed to process request",
                "system",
            ))
        })
    }

    async fn handle_request(request: Request) -> Result<Response, ApiError> {
        // Implement actual request handling logic
        // This is a placeholder for the actual implementation
        Ok(Response::default())
    }

    async fn shutdown(&self) {
        // Signal the worker to stop processing new requests
        self.request_tx.closed().await;
        
        // Wait for any in-progress requests to complete
        while self.metrics.queue_size.get() > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub id: String,
    // Add other request fields
}

#[derive(Debug, Default)]
pub struct Response {
    // Add response fields
}

#[derive(Debug, Default)]
struct QueueMetrics {
    total_size: usize,
    avg_queue_size: f64,
    std_dev: f64,
    max_size: usize,
}

#[derive(Debug, Default)]
struct SystemMetrics {
    cpu_usage: f64,
    memory_usage: f64,
    gpu_usage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_worker_pool_scaling() {
        let config = Arc::new(AppConfig {
            max_concurrent_requests: 1000,
            min_workers: 5,
            worker_queue_size: 100,
            load_check_interval: Duration::from_secs(1),
            host: "127.0.0.1".to_string(),
            port: 8080,
            rate_limit_requests_per_second: 1000,
            num_workers: 5,
        });

        let pool = AdaptiveWorkerPool::new(config).await;
        
        // Initial worker count should match min_workers
        assert_eq!(pool.workers.len(), 5);

        // Simulate high load
        for worker in &pool.workers {
            for _ in 0..50 {
                worker.metrics.queue_size.inc();
            }
        }

        // Wait for scaling to occur
        sleep(Duration::from_secs(2)).await;
        pool.adjust_worker_count().await;

        // Worker count should have increased
        assert!(pool.workers.len() > 5);

        // Simulate low load
        for worker in &pool.workers {
            while worker.metrics.queue_size.get() > 0 {
                worker.metrics.queue_size.dec();
            }
        }

        // Wait for scaling to occur
        sleep(Duration::from_secs(2)).await;
        pool.adjust_worker_count().await;

        // Worker count should have decreased
        assert_eq!(pool.workers.len(), 5);
    }
} 