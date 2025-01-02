use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec,
};
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::tenant::Tenant,
    services::tenant_service::TenantService,
};

lazy_static::lazy_static! {
    static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests",
        &["tenant_id", "method", "path", "status"]
    ).unwrap();

    static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["tenant_id", "method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    static ref GPU_TIME_SECONDS: HistogramVec = register_histogram_vec!(
        "gpu_time_seconds",
        "GPU processing time in seconds",
        &["tenant_id", "operation"],
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0]
    ).unwrap();

    static ref DATA_PROCESSED_BYTES: IntCounterVec = register_int_counter_vec!(
        "data_processed_bytes",
        "Total amount of data processed in bytes",
        &["tenant_id", "direction"]
    ).unwrap();
}

pub struct MetricsMiddleware {
    tenant_service: Arc<TenantService>,
}

impl MetricsMiddleware {
    pub fn new(tenant_service: Arc<TenantService>) -> Self {
        Self { tenant_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(MetricsMiddlewareService {
            service: Rc::new(service),
            tenant_service: self.tenant_service.clone(),
        }))
    }
}

pub struct MetricsMiddlewareService<S> {
    service: Rc<S>,
    tenant_service: Arc<TenantService>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let tenant_service = self.tenant_service.clone();
        let start_time = Instant::now();

        // Extract request information
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let content_length = req
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        Box::pin(async move {
            // Get tenant from request extensions (set by TenantMiddleware)
            let tenant = req
                .extensions()
                .get::<Tenant>()
                .ok_or_else(|| Error::from(ServiceError::InvalidRequest(
                    "Tenant not found in request extensions".to_string(),
                )))?;

            let tenant_id = tenant.id.to_string();

            // Record request size
            DATA_PROCESSED_BYTES
                .with_label_values(&[&tenant_id, "in"])
                .inc_by(content_length as u64);

            // Call the next service
            let res = service.call(req).await?;

            // Record metrics
            let status = res.status().as_u16().to_string();
            let duration = start_time.elapsed().as_secs_f64();

            HTTP_REQUESTS_TOTAL
                .with_label_values(&[&tenant_id, &method, &path, &status])
                .inc();

            HTTP_REQUEST_DURATION_SECONDS
                .with_label_values(&[&tenant_id, &method, &path])
                .observe(duration);

            // Record response size
            if let Some(content_length) = res
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<i64>().ok())
            {
                DATA_PROCESSED_BYTES
                    .with_label_values(&[&tenant_id, "out"])
                    .inc_by(content_length as u64);
            }

            // Record GPU time if present
            if let Some(gpu_time) = res
                .headers()
                .get("X-GPU-Time-Ms")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<f64>().ok())
            {
                GPU_TIME_SECONDS
                    .with_label_values(&[&tenant_id, "inferencing"])
                    .observe(gpu_time / 1000.0);
            }

            Ok(res)
        })
    }
} 