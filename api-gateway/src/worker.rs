use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicUsize, AtomicU64};
use std::sync::atomic::Ordering;

use crate::config::AppConfig;
use crate::error::{ApiError, ErrorCode, ErrorDetails};
use crate::metrics::WorkerMetrics;

pub struct Worker {
    id: usize,
    request_tx: mpsc::Sender<Request>,
    metrics: Arc<WorkerMetrics>,
    config: Arc<AppConfig>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug)]
pub struct Request {
    pub id: Uuid,
    pub payload: Vec<u8>,
    pub response_tx: mpsc::Sender<Result<Response, ApiError>>,
}

#[derive(Debug)]
pub struct Response {
    pub id: Uuid,
    pub payload: Vec<u8>,
}

impl Worker {
    pub fn new(
        id: usize,
        semaphore: Arc<Semaphore>,
        metrics: Arc<WorkerMetrics>,
        config: Arc<AppConfig>,
    ) -> Self {
        let (request_tx, mut request_rx) = mpsc::channel(config.worker_queue_size);
        
        let worker_metrics = Arc::clone(&metrics);
        let worker_config = Arc::clone(&config);
        let worker_semaphore = Arc::clone(&semaphore);
        
        tokio::spawn(async move {
            info!("Worker {} started", id);
            
            while let Some(request) = request_rx.recv().await {
                let _permit = match worker_semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        error!("Worker {} failed to acquire semaphore: {}", id, e);
                        let error = ApiError::new(
                            ErrorCode::ResourceExhausted,
                            "Failed to acquire worker permit".to_string(),
                        ).with_details(ErrorDetails {
                            error_type: "SemaphoreError".to_string(),
                            source: Some(e.to_string()),
                            stack_trace: None,
                            correlation_id: Some(request.id.to_string()),
                            metadata: None,
                        });
                        
                        if let Err(e) = request.response_tx.send(Err(error)).await {
                            error!("Failed to send error response: {}", e);
                        }
                        continue;
                    }
                };
                
                worker_metrics.active_requests.inc();
                
                let result = match Self::process_request(request.payload).await {
                    Ok(response_payload) => {
                        worker_metrics.successful_requests.inc();
                        Ok(Response {
                            id: request.id,
                            payload: response_payload,
                        })
                    }
                    Err(e) => {
                        worker_metrics.failed_requests.inc();
                        Err(e)
                    }
                };
                
                if let Err(e) = request.response_tx.send(result).await {
                    error!("Worker {} failed to send response: {}", id, e);
                    worker_metrics.failed_responses.inc();
                }
                
                worker_metrics.active_requests.dec();
            }
            
            info!("Worker {} shutting down", id);
        });
        
        Self {
            id,
            request_tx,
            metrics,
            config,
            semaphore,
        }
    }
    
    async fn process_request(payload: Vec<u8>) -> Result<Vec<u8>, ApiError> {
        // Validate request size
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(ApiError::new(
                ErrorCode::PayloadTooLarge,
                format!("Payload size {} exceeds maximum of {}", payload.len(), MAX_PAYLOAD_SIZE),
            ).with_retry_after(1));
        }

        // Validate request content
        if payload.is_empty() {
            return Err(ApiError::new(
                ErrorCode::BadRequest,
                "Empty request payload".to_string(),
            ));
        }

        // Parse and validate request format
        let request_data = match serde_json::from_slice::<serde_json::Value>(&payload) {
            Ok(data) => data,
            Err(e) => {
                return Err(ApiError::new(
                    ErrorCode::DataValidationFailed,
                    "Invalid JSON payload".to_string(),
                ).with_details(ErrorDetails {
                    error_type: "JsonParseError".to_string(),
                    source: Some(e.to_string()),
                    stack_trace: None,
                    correlation_id: None,
                    metadata: None,
                }));
            }
        };

        // Process request with timeout
        let processing_result = tokio::time::timeout(
            Duration::from_secs(30),
            Self::execute_request(request_data)
        ).await;

        match processing_result {
            Ok(result) => result,
            Err(_) => Err(ApiError::new(
                ErrorCode::RequestTimeout,
                "Request processing timed out".to_string(),
            ).with_retry_after(5)),
        }
    }

    async fn execute_request(data: serde_json::Value) -> Result<Vec<u8>, ApiError> {
        // Add circuit breaker pattern
        static FAILURES: AtomicUsize = AtomicUsize::new(0);
        static LAST_FAILURE: AtomicU64 = AtomicU64::new(0);
        
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let last_failure = LAST_FAILURE.load(Ordering::Relaxed);
        let failure_count = FAILURES.load(Ordering::Relaxed);
        
        // Check circuit breaker
        if failure_count >= 5 && current_time - last_failure < 60 {
            return Err(ApiError::new(
                ErrorCode::CircuitBreakerOpen,
                "Service temporarily unavailable due to high error rate".to_string(),
            ).with_retry_after(60 - (current_time - last_failure)));
        }

        // Process the request with retries
        let mut retries = 0;
        let max_retries = 3;
        
        loop {
            match Self::do_execute_request(&data).await {
                Ok(result) => {
                    // Reset circuit breaker on success
                    if failure_count > 0 {
                        FAILURES.store(0, Ordering::Relaxed);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if retries >= max_retries {
                        // Update circuit breaker
                        FAILURES.fetch_add(1, Ordering::Relaxed);
                        LAST_FAILURE.store(current_time, Ordering::Relaxed);
                        return Err(e);
                    }
                    
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2u64.pow(retries as u32));
                    tokio::time::sleep(delay).await;
                    retries += 1;
                    
                    warn!(
                        "Request failed, retrying {}/{}: {}",
                        retries, max_retries, e
                    );
                }
            }
        }
    }

    async fn do_execute_request(data: &serde_json::Value) -> Result<Vec<u8>, ApiError> {
        // Simulate processing with random delay and potential failures
        let processing_time = rand::random::<u64>() % 100;
        tokio::time::sleep(Duration::from_millis(processing_time)).await;

        // Simulate random failures for testing
        if rand::random::<f64>() < 0.01 {
            return Err(ApiError::new(
                ErrorCode::InternalError,
                "Simulated random failure".to_string(),
            ));
        }

        // Process the request (placeholder for actual processing logic)
        Ok(serde_json::to_vec(&data).map_err(|e| ApiError::new(
            ErrorCode::DataCorrupted,
            format!("Failed to serialize response: {}", e),
        ))?)
    }

    pub async fn handle_request(&self, request: Request) -> Result<(), ApiError> {
        // Check worker health
        if !self.is_healthy().await {
            return Err(ApiError::new(
                ErrorCode::WorkerNotResponding,
                "Worker is not healthy".to_string(),
            ));
        }

        // Check queue capacity with headroom
        let queue_size = self.metrics.queue_size.get();
        let max_size = self.config.worker_queue_size as i64;
        
        if queue_size >= max_size * 9 / 10 {  // 90% full
            return Err(ApiError::new(
                ErrorCode::WorkerPoolFull,
                "Worker queue is near capacity".to_string(),
            ).with_retry_after(1));
        }

        // Try to send request to worker
        match self.request_tx.try_send(request) {
            Ok(_) => {
                self.metrics.queue_size.inc();
                debug!("Request queued for worker {}", self.id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to queue request for worker {}: {}", self.id, e);
                Err(ApiError::new(
                    ErrorCode::WorkerError,
                    format!("Failed to queue request: {}", e),
                ).with_details(ErrorDetails {
                    error_type: "QueueError".to_string(),
                    source: Some(e.to_string()),
                    stack_trace: None,
                    correlation_id: None,
                    metadata: None,
                }))
            }
        }
    }

    async fn is_healthy(&self) -> bool {
        // Check if worker is processing requests within expected timeframes
        let active_requests = self.metrics.active_requests.get();
        let queue_size = self.metrics.queue_size.get();
        let error_rate = self.metrics.failed_requests.get() as f64 / 
                        self.metrics.successful_requests.get().max(1) as f64;

        // Worker is considered unhealthy if:
        // 1. It has too many active requests
        // 2. Its queue is full
        // 3. It has a high error rate
        active_requests < 100 && 
        queue_size < self.config.worker_queue_size as i64 &&
        error_rate < 0.1
    }

    pub async fn shutdown(&self) {
        info!("Initiating shutdown for worker {}", self.id);
        
        // Close the channel to stop accepting new requests
        drop(self.request_tx.clone());
        
        // Wait for queue to drain
        while self.metrics.queue_size.get() > 0 {
            warn!(
                "Worker {} still has {} requests in queue",
                self.id,
                self.metrics.queue_size.get()
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        info!("Worker {} shutdown complete", self.id);
    }
}

const MAX_PAYLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    
    #[tokio::test]
    async fn test_worker_processing() {
        let config = Arc::new(AppConfig::default());
        let metrics = Arc::new(WorkerMetrics::new());
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let worker = Worker::new(1, semaphore, metrics.clone(), config);
        
        let (response_tx, mut response_rx) = mpsc::channel(1);
        let request = Request {
            id: Uuid::new_v4(),
            payload: vec![1, 2, 3],
            response_tx,
        };
        
        assert!(worker.handle_request(request).await.is_ok());
        
        let response = response_rx.recv().await.unwrap();
        assert!(response.is_ok());
        assert_eq!(response.unwrap().payload, vec![1, 2, 3]);
    }
    
    #[tokio::test]
    async fn test_worker_error_handling() {
        let config = Arc::new(AppConfig::default());
        let metrics = Arc::new(WorkerMetrics::new());
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let worker = Worker::new(1, semaphore, metrics.clone(), config);
        
        let (response_tx, mut response_rx) = mpsc::channel(1);
        let request = Request {
            id: Uuid::new_v4(),
            payload: vec![], // Empty payload should trigger an error
            response_tx,
        };
        
        assert!(worker.handle_request(request).await.is_ok());
        
        let response = response_rx.recv().await.unwrap();
        assert!(response.is_err());
        
        let error = response.unwrap_err();
        assert_eq!(error.code, ErrorCode::BadRequest);
    }
    
    #[tokio::test]
    async fn test_worker_queue_full() {
        let mut config = AppConfig::default();
        config.worker_queue_size = 1;
        let config = Arc::new(config);
        
        let metrics = Arc::new(WorkerMetrics::new());
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let worker = Worker::new(1, semaphore, metrics.clone(), config);
        
        // Fill the queue
        let (response_tx1, _) = mpsc::channel(1);
        let request1 = Request {
            id: Uuid::new_v4(),
            payload: vec![1],
            response_tx: response_tx1,
        };
        
        assert!(worker.handle_request(request1).await.is_ok());
        
        // Try to queue another request
        let (response_tx2, _) = mpsc::channel(1);
        let request2 = Request {
            id: Uuid::new_v4(),
            payload: vec![2],
            response_tx: response_tx2,
        };
        
        let result = worker.handle_request(request2).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode::WorkerPoolFull);
        assert!(error.retry_after.is_some());
    }
} 