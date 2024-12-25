use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;
use serde_json::json;

use crate::worker_pool::{Request, Response, AdaptiveWorkerPool};
use crate::config::AppConfig;
use crate::metrics::WorkerMetrics;

pub struct TestRequest {
    pub endpoint: String,
    pub method: String,
    pub payload: serde_json::Value,
}

impl TestRequest {
    pub fn new(endpoint: &str, method: &str, payload: serde_json::Value) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            method: method.to_string(),
            payload,
        }
    }

    pub fn to_request(&self) -> Request {
        Request {
            id: Uuid::new_v4().to_string(),
            endpoint: self.endpoint.clone(),
            method: self.method.clone(),
            payload: self.payload.clone(),
            timestamp: Instant::now(),
        }
    }
}

pub struct LoadTestConfig {
    pub concurrency: usize,
    pub duration: Duration,
    pub ramp_up: Duration,
    pub ramp_down: Duration,
    pub request_generator: Box<dyn Fn() -> TestRequest + Send + Sync>,
}

pub struct LoadTestResults {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub avg_response_time: f64,
    pub p95_response_time: f64,
    pub p99_response_time: f64,
    pub errors: Vec<String>,
}

pub async fn run_load_test(
    worker_pool: Arc<AdaptiveWorkerPool>,
    config: LoadTestConfig,
) -> LoadTestResults {
    let start_time = Instant::now();
    let concurrency_semaphore = Arc::new(Semaphore::new(config.concurrency));
    let mut response_times = Vec::new();
    let mut errors = Vec::new();
    let mut total_requests = 0;
    let mut successful_requests = 0;
    let mut failed_requests = 0;

    // Calculate the number of requests to add during ramp-up
    let ramp_up_interval = if config.ramp_up.as_secs() > 0 {
        Duration::from_secs_f64(config.ramp_up.as_secs_f64() / config.concurrency as f64)
    } else {
        Duration::from_secs(0)
    };

    // Spawn concurrent request generators
    let mut handles = Vec::new();
    for i in 0..config.concurrency {
        let semaphore = Arc::clone(&concurrency_semaphore);
        let worker_pool = Arc::clone(&worker_pool);
        let request_generator = Arc::new(config.request_generator);
        
        // Add delay for ramp-up
        let start_delay = ramp_up_interval * i as u32;

        let handle = tokio::spawn(async move {
            let mut local_response_times = Vec::new();
            let mut local_errors = Vec::new();
            let mut local_total = 0;
            let mut local_success = 0;
            let mut local_failed = 0;

            // Wait for ramp-up delay
            if start_delay > Duration::from_secs(0) {
                tokio::time::sleep(start_delay).await;
            }

            while start_time.elapsed() < config.duration {
                let _permit = semaphore.acquire().await.unwrap();
                let request = request_generator().to_request();
                let start = Instant::now();
                
                match worker_pool.process_request(request).await {
                    Ok(_) => {
                        local_response_times.push(start.elapsed().as_secs_f64());
                        local_success += 1;
                    }
                    Err(e) => {
                        local_errors.push(e.to_string());
                        local_failed += 1;
                    }
                }
                
                local_total += 1;
            }

            (
                local_response_times,
                local_errors,
                local_total,
                local_success,
                local_failed,
            )
        });

        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (local_times, local_errors, local_total, local_success, local_failed) =
            handle.await.unwrap();
        response_times.extend(local_times);
        errors.extend(local_errors);
        total_requests += local_total;
        successful_requests += local_success;
        failed_requests += local_failed;
    }

    // Calculate statistics
    response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
    let p95_index = (response_times.len() as f64 * 0.95) as usize;
    let p99_index = (response_times.len() as f64 * 0.99) as usize;

    LoadTestResults {
        total_requests,
        successful_requests,
        failed_requests,
        avg_response_time,
        p95_response_time: response_times[p95_index],
        p99_response_time: response_times[p99_index],
        errors,
    }
}

// Helper function to create a test worker pool
pub async fn create_test_worker_pool(concurrency: usize, min_workers: usize) -> Arc<AdaptiveWorkerPool> {
    let config = Arc::new(AppConfig {
        max_concurrent_requests: concurrency,
        min_workers,
        worker_queue_size: 1000,
        load_check_interval: Duration::from_secs(5),
        host: "127.0.0.1".to_string(),
        port: 8080,
        rate_limit_requests_per_second: 1000,
        num_workers: min_workers,
    });

    Arc::new(AdaptiveWorkerPool::new(config).await)
}

// Helper function to generate test payloads
pub fn generate_bot_creation_payload() -> serde_json::Value {
    json!({
        "name": format!("test-bot-{}", Uuid::new_v4()),
        "type": "support",
        "description": "Test bot for load testing",
        "configuration": {
            "language": "en",
            "response_time": "fast",
            "max_tokens": 1000
        }
    })
}

pub fn generate_bot_execution_payload(bot_id: &str) -> serde_json::Value {
    json!({
        "bot_id": bot_id,
        "input": {
            "message": "Test message for load testing",
            "context": {
                "user_id": Uuid::new_v4().to_string(),
                "session_id": Uuid::new_v4().to_string()
            }
        },
        "parameters": {
            "temperature": 0.7,
            "max_tokens": 500
        }
    })
} 