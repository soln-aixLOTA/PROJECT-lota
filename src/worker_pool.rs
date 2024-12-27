use crate::config::AppConfig;
use crate::error::ApiError;
use dashmap::DashMap;
use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use serde::{Deserialize, Serialize};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, Semaphore},
    time::sleep,
};
use tracing::error;

lazy_static! {
    static ref WORKER_METRICS: IntCounterVec = register_int_counter_vec!(
        "api_gateway_worker_metrics",
        "Worker pool metrics",
        &["worker_id", "metric_type"]
    )
    .unwrap();
    static ref WORKER_LATENCY: HistogramVec = register_histogram_vec!(
        "api_gateway_worker_latency_seconds",
        "Worker processing latency in seconds",
        &["worker_id"],
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]
    )
    .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub payload: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub result: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct AdaptiveWorkerPool {
    config: Arc<AppConfig>,
    workers: Arc<DashMap<String, Worker>>,
    semaphore: Arc<Semaphore>,
}

#[derive(Clone)]
struct Worker {
    id: String,
    sender: mpsc::Sender<Request>,
    last_used: Instant,
    metrics: WorkerMetrics,
}

#[derive(Debug, Clone)]
struct WorkerMetrics {
    requests_processed: u64,
    errors: u64,
    avg_processing_time: Duration,
}

impl AdaptiveWorkerPool {
    pub async fn new(config: AppConfig) -> Self {
        let pool = AdaptiveWorkerPool {
            config: Arc::new(config),
            workers: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(10)), // Default max concurrent requests
        };

        // Start metrics collection
        pool.clone().start_metrics_collection();

        pool
    }

    pub async fn process_request(&self, request: Request) -> Result<Response, ApiError> {
        let permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to acquire semaphore"))?;

        let worker = self.get_available_worker().await?;
        let response = worker.process_request(request).await;

        drop(permit);
        response
    }

    async fn get_available_worker(&self) -> Result<Arc<Worker>, ApiError> {
        // Simple round-robin for now
        let workers = self.workers.iter();
        if let Some(worker) = workers.min_by_key(|w| w.value().last_used) {
            Ok(Arc::new(worker.value().clone()))
        } else {
            // Create new worker if none available
            self.create_worker().await
        }
    }

    async fn create_worker(&self) -> Result<Arc<Worker>, ApiError> {
        let worker_id = uuid::Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(100);

        let worker = Worker {
            id: worker_id.clone(),
            sender: tx,
            last_used: Instant::now(),
            metrics: WorkerMetrics {
                requests_processed: 0,
                errors: 0,
                avg_processing_time: Duration::from_secs(0),
            },
        };

        // Start worker processing loop
        let worker_clone = worker.clone();
        let worker_id_clone = worker_id.clone();
        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                let start = Instant::now();
                match worker_clone.handle_request(request).await {
                    Ok(_) => {
                        WORKER_METRICS
                            .with_label_values(&[&worker_id_clone, "success"])
                            .inc();
                    }
                    Err(e) => {
                        error!("Worker error: {}", e);
                        WORKER_METRICS
                            .with_label_values(&[&worker_id_clone, "error"])
                            .inc();
                    }
                }
                WORKER_LATENCY
                    .with_label_values(&[&worker_id_clone])
                    .observe(start.elapsed().as_secs_f64());
            }
        });

        self.workers.insert(worker_id, worker.clone());
        Ok(Arc::new(worker))
    }

    fn start_metrics_collection(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                self.collect_metrics().await;
            }
        });
    }

    async fn collect_metrics(&self) {
        for worker in self.workers.iter() {
            WORKER_METRICS
                .with_label_values(&[&worker.id, "total_requests"])
                .inc_by(worker.metrics.requests_processed);
            WORKER_METRICS
                .with_label_values(&[&worker.id, "errors"])
                .inc_by(worker.metrics.errors);
        }
    }
}

impl Worker {
    async fn process_request(&self, request: Request) -> Result<Response, ApiError> {
        let start = Instant::now();

        // Send request to worker channel
        self.sender
            .send(request.clone())
            .await
            .map_err(|_| ApiError::new("INTERNAL_ERROR", "Failed to send request to worker"))?;

        // For now, just echo back the request as response
        let response = Response {
            id: request.id,
            result: request.payload,
            metadata: Some(serde_json::json!({
                "worker_id": self.id,
                "processing_time": start.elapsed().as_secs_f64()
            })),
        };

        Ok(response)
    }

    async fn handle_request(&self, _request: Request) -> Result<(), ApiError> {
        // Simulate some work
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}
