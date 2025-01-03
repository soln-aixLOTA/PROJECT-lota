use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use dashmap::DashMap;
use futures::future::{ok, Future, Ready};
use prometheus::{register_counter_vec, register_gauge_vec, CounterVec, GaugeVec};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Instant,
};
use tracing::{error, info};
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref REQUEST_METRICS: CounterVec = register_counter_vec!(
        "api_request_metrics",
        "API request metrics by tenant",
        &["tenant_id", "metric"]
    ).unwrap();

    static ref RESOURCE_METRICS: GaugeVec = register_gauge_vec!(
        "api_resource_metrics",
        "API resource usage metrics by tenant",
        &["tenant_id", "metric"]
    ).unwrap();

    static ref TENANT_USAGE: DashMap<Uuid, TenantUsage> = DashMap::new();
}

#[derive(Debug, Clone)]
pub struct TenantUsage {
    pub request_count: u64,
    pub data_processed: u64,
    pub processing_time: u64,
    pub gpu_time: u64,
}

impl Default for TenantUsage {
    fn default() -> Self {
        Self {
            request_count: 0,
            data_processed: 0,
            processing_time: 0,
            gpu_time: 0,
        }
    }
}

pub fn record_api_usage(
    tenant_id: Uuid,
    processing_time_ms: u64,
    gpu_time_ms: u64,
    data_size_bytes: u64,
) {
    let mut usage = TENANT_USAGE
        .entry(tenant_id)
        .or_insert_with(TenantUsage::default);

    usage.request_count += 1;
    usage.processing_time += processing_time_ms;
    usage.gpu_time += gpu_time_ms;
    usage.data_processed += data_size_bytes;

    REQUEST_METRICS
        .with_label_values(&[&tenant_id.to_string(), "request_count"])
        .inc(1);
    RESOURCE_METRICS
        .with_label_values(&[&tenant_id.to_string(), "processing_time_ms"])
        .set(processing_time_ms as f64);
    RESOURCE_METRICS
        .with_label_values(&[&tenant_id.to_string(), "gpu_time_ms"])
        .set(gpu_time_ms as f64);
    RESOURCE_METRICS
        .with_label_values(&[&tenant_id.to_string(), "data_processed_bytes"])
        .set(data_size_bytes as f64);
}

pub fn get_tenant_usage(tenant_id: Uuid) -> Option<TenantUsage> {
    TENANT_USAGE.get(&tenant_id).map(|usage| usage.clone())
}

pub struct UsageTrackingMiddleware;

impl UsageTrackingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for UsageTrackingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UsageTrackingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UsageTrackingMiddlewareService { service })
    }
}

pub struct UsageTrackingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for UsageTrackingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let tenant_id = req
            .extensions()
            .get::<Uuid>()
            .cloned()
            .unwrap_or_else(|| Uuid::nil());

        let fut = self.service.call(req);

        Box::pin(async move {
            match fut.await {
                Ok(res) => {
                    let duration = start_time.elapsed();
                    let processing_time_ms = duration.as_millis() as u64;

                    // Simulated GPU time - in a real application, this would come from the actual processing
                    let gpu_time_ms = processing_time_ms / 2;

                    // Simulated data size - in a real application, this would be the actual request/response size
                    let data_size_bytes = 1000;

                    record_api_usage(tenant_id, processing_time_ms, gpu_time_ms, data_size_bytes);

                    info!(
                        "Request processed - Tenant: {}, Time: {}ms, GPU Time: {}ms, Data: {} bytes",
                        tenant_id, processing_time_ms, gpu_time_ms, data_size_bytes
                    );

                    Ok(res)
                }
                Err(e) => {
                    error!(
                        "Error processing request - Tenant: {}, Error: {}",
                        tenant_id,
                        e.to_string()
                    );
                    Err(e)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_usage_tracking() {
        let tenant_id = Uuid::new_v4();

        // Record some usage
        record_api_usage(tenant_id, 100, 50, 1000);

        // Simulate some delay
        sleep(Duration::from_millis(10)).await;

        // Record more usage
        record_api_usage(tenant_id, 200, 100, 2000);

        // Get the total usage
        if let Some(usage) = get_tenant_usage(tenant_id) {
            assert_eq!(usage.request_count, 2);
            assert_eq!(usage.processing_time, 300);
            assert_eq!(usage.gpu_time, 150);
            assert_eq!(usage.data_processed, 3000);
        } else {
            panic!("Usage tracking failed");
        }
    }
}
