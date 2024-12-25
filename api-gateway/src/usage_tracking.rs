use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;
use metrics::{counter, gauge};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub request_count: u64,
    pub total_processing_time_ms: u64,
    pub gpu_time_ms: u64,
    pub data_processed_bytes: u64,
    pub last_request: DateTime<Utc>,
}

impl Default for UsageMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            total_processing_time_ms: 0,
            gpu_time_ms: 0,
            data_processed_bytes: 0,
            last_request: Utc::now(),
        }
    }
}

#[derive(Clone)]
pub struct UsageTracker {
    // Using DashMap for better concurrent performance
    metrics: Arc<DashMap<String, UsageMetrics>>,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
        }
    }

    pub async fn record_request(
        &self,
        tenant_id: &str,
        processing_time_ms: u64,
        gpu_time_ms: u64,
        data_size_bytes: u64,
    ) {
        // Update metrics atomically using DashMap's entry API
        self.metrics
            .entry(tenant_id.to_string())
            .and_modify(|metrics| {
                metrics.request_count += 1;
                metrics.total_processing_time_ms += processing_time_ms;
                metrics.gpu_time_ms += gpu_time_ms;
                metrics.data_processed_bytes += data_size_bytes;
                metrics.last_request = Utc::now();
                
                // Update Prometheus metrics
                counter!("request_count", "tenant" => tenant_id.to_string()).increment(1);
                gauge!("processing_time_ms", "tenant" => tenant_id.to_string()).set(processing_time_ms as f64);
                gauge!("gpu_time_ms", "tenant" => tenant_id.to_string()).set(gpu_time_ms as f64);
                gauge!("data_processed_bytes", "tenant" => tenant_id.to_string()).set(data_size_bytes as f64);
            })
            .or_insert_with(|| {
                let metrics = UsageMetrics {
                    request_count: 1,
                    total_processing_time_ms: processing_time_ms,
                    gpu_time_ms,
                    data_processed_bytes: data_size_bytes,
                    last_request: Utc::now(),
                };
                
                // Initialize Prometheus metrics
                counter!("request_count", "tenant" => tenant_id.to_string()).increment(1);
                gauge!("processing_time_ms", "tenant" => tenant_id.to_string()).set(processing_time_ms as f64);
                gauge!("gpu_time_ms", "tenant" => tenant_id.to_string()).set(gpu_time_ms as f64);
                gauge!("data_processed_bytes", "tenant" => tenant_id.to_string()).set(data_size_bytes as f64);
                
                metrics
            });
    }

    pub fn get_metrics(&self, tenant_id: &str) -> Option<UsageMetrics> {
        self.metrics.get(tenant_id).map(|r| r.clone())
    }

    pub fn get_all_metrics(&self) -> HashMap<String, UsageMetrics> {
        self.metrics
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    // Export metrics for billing system with efficient iteration
    pub fn export_billing_metrics(&self) -> Vec<BillingRecord> {
        self.metrics
            .iter()
            .map(|entry| BillingRecord {
                tenant_id: entry.key().clone(),
                timestamp: Utc::now(),
                request_count: entry.value().request_count,
                gpu_time_ms: entry.value().gpu_time_ms,
                data_processed_bytes: entry.value().data_processed_bytes,
            })
            .collect()
    }

    // Cleanup old metrics to prevent memory growth
    pub async fn cleanup_old_metrics(&self, retention_period: chrono::Duration) {
        let cutoff = Utc::now() - retention_period;
        self.metrics.retain(|_, metrics| metrics.last_request > cutoff);
    }
}

#[derive(Debug, Serialize)]
pub struct BillingRecord {
    pub tenant_id: String,
    pub timestamp: DateTime<Utc>,
    pub request_count: u64,
    pub gpu_time_ms: u64,
    pub data_processed_bytes: u64,
}

// Middleware for tracking usage
pub struct UsageTrackingMiddleware {
    tracker: UsageTracker,
}

impl UsageTrackingMiddleware {
    pub fn new(tracker: UsageTracker) -> Self {
        Self { tracker }
    }
}

impl<S> Transform<S, ServiceRequest> for UsageTrackingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = UsageTrackingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UsageTrackingMiddlewareService {
            service,
            tracker: self.tracker.clone(),
        })
    }
}

pub struct UsageTrackingMiddlewareService<S> {
    service: S,
    tracker: UsageTracker,
}

impl<S> Service<ServiceRequest> for UsageTrackingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let tenant_id = req
            .headers()
            .get("X-Tenant-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("anonymous")
            .to_string();

        let tracker = self.tracker.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let response = fut.await?;
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            // Extract GPU time and data size from response headers
            let gpu_time = response
                .headers()
                .get("X-GPU-Time-MS")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let data_size = response
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            tracker
                .record_request(&tenant_id, processing_time, gpu_time, data_size)
                .await;

            Ok(response)
        })
    }
} 