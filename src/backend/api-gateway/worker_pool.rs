use dashmap::DashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{mpsc, Semaphore},
    time::sleep,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{config::AppConfig, error::ApiError};

/// A request that can be processed by a worker
#[derive(Debug, Clone)]
pub struct Request {
    /// Unique identifier for the request
    pub id: String,
    /// The payload to be processed
    pub payload: Vec<u8>,
    /// Request metadata
    pub metadata: Option<serde_json::Value>,
}

/// A response from a worker after processing a request
#[derive(Debug, Clone)]
pub struct Response {
    /// Unique identifier matching the request
    pub id: String,
    /// The processed result
    pub result: Vec<u8>,
    /// Response metadata
    pub metadata: Option<serde_json::Value>,
}

/// Manages a pool of workers for processing requests
/// Automatically scales the number of workers based on load
#[derive(Clone)]
pub struct AdaptiveWorkerPool {
    /// Application configuration
    config: Arc<AppConfig>,
    /// Active workers in the pool
    workers: Arc<DashMap<String, Worker>>,
    /// Controls the maximum number of concurrent workers
    semaphore: Arc<Semaphore>,
}

/// A worker that can process requests
#[derive(Clone)]
struct Worker {
    /// Unique identifier for the worker
    id: String,
    /// Channel for sending requests to the worker
    sender: mpsc::Sender<Request>,
    /// Last time the worker was used
    last_used: Instant,
}

/// Metrics for monitoring worker performance
#[derive(Debug, Clone)]
struct WorkerMetrics {
    /// Number of requests processed
    requests_processed: u64,
    /// Number of errors encountered
    errors: u64,
    /// Average processing time per request
    avg_processing_time: Duration,
}

impl AdaptiveWorkerPool {
    /// Creates a new worker pool
    ///
    /// # Arguments
    /// * `config` - Application configuration
    pub async fn new(config: AppConfig) -> Self {
        info!("Initializing adaptive worker pool");
        AdaptiveWorkerPool {
            config: Arc::new(config),
            workers: Arc::new(DashMap::new()),
            semaphore: Arc::new(Semaphore::new(10)), // Default max workers
        }
    }

    /// Processes a request using an available worker
    ///
    /// # Arguments
    /// * `request` - The request to process
    ///
    /// # Returns
    /// * `Result<Response, ApiError>` - The processed response or an error
    pub async fn process_request(&self, request: Request) -> Result<Response, ApiError> {
        info!("Processing request: {}", request.id);

        // Get an available worker
        let worker = self.get_available_worker().await?;

        // Send request to worker
        if let Err(e) = worker.sender.send(request.clone()).await {
            error!("Failed to send request to worker: {}", e);
            return Err(ApiError::internal_error());
        }

        // TODO: Implement response handling
        Ok(Response {
            id: request.id,
            result: vec![],
            metadata: None,
        })
    }

    /// Gets an available worker or creates a new one if needed
    async fn get_available_worker(&self) -> Result<Arc<Worker>, ApiError> {
        // Try to find an available worker
        for worker in self.workers.iter() {
            if worker.last_used.elapsed() < Duration::from_secs(60) {
                return Ok(Arc::new(worker.clone()));
            }
        }

        // Create a new worker if under limit
        if let Ok(_permit) = self.semaphore.try_acquire() {
            return self.create_worker().await;
        }

        warn!("Worker pool at capacity");
        Err(ApiError::service_unavailable())
    }

    /// Creates a new worker
    async fn create_worker(&self) -> Result<Arc<Worker>, ApiError> {
        let worker_id = Uuid::new_v4().to_string();
        info!("Creating new worker: {}", worker_id);

        let (tx, mut rx) = mpsc::channel(100);
        let worker = Worker {
            id: worker_id.clone(),
            sender: tx,
            last_used: Instant::now(),
        };

        // Spawn worker task
        let worker_clone = worker.clone();
        let worker_id_for_task = worker_id.clone(); // Clone for the async block
        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if let Err(e) = worker_clone.handle_request(request).await {
                    error!(
                        "Worker {} failed to process request: {}",
                        worker_id_for_task, e
                    );
                }
            }
        });

        let worker = Arc::new(worker);
        self.workers.insert(worker_id.clone(), (*worker).clone());

        Ok(worker)
    }
}

impl Worker {
    /// Processes a request
    async fn process_request(&self, request: Request) -> Result<Response, ApiError> {
        info!("Worker {} processing request {}", self.id, request.id);

        // Simulate processing time
        sleep(Duration::from_millis(100)).await;

        Ok(Response {
            id: request.id,
            result: request.payload,
            metadata: request.metadata,
        })
    }

    /// Handles an incoming request
    async fn handle_request(&self, request: Request) -> Result<(), ApiError> {
        match self.process_request(request).await {
            Ok(_response) => {
                info!("Worker {} successfully processed request", self.id);
                Ok(())
            }
            Err(e) => {
                error!("Worker {} failed to process request: {}", self.id, e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_pool_creation() {
        let config = AppConfig::default();
        let pool = AdaptiveWorkerPool::new(config).await;
        assert_eq!(pool.workers.len(), 0);
    }

    #[tokio::test]
    async fn test_request_processing() {
        let config = AppConfig::default();
        let pool = AdaptiveWorkerPool::new(config).await;

        let request = Request {
            id: Uuid::new_v4().to_string(),
            payload: vec![1, 2, 3],
            metadata: None,
        };

        let response = pool.process_request(request.clone()).await;
        assert!(response.is_ok());
    }
}
