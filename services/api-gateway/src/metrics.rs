use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Future, Ready};
use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, Histogram, HistogramVec, IntCounterVec,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};
use tracing::{error, info};
use uuid::Uuid;

lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests made.",
        &["method", "path", "status"]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds.",
        &["method", "path"],
        vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_SIZE_BYTES: HistogramVec = register_histogram_vec!(
        "http_request_size_bytes",
        "HTTP request size in bytes.",
        &["method", "path"],
        exponential_buckets(100.0, 2.0, 10).unwrap()
    )
    .unwrap();
    pub static ref HTTP_RESPONSE_SIZE_BYTES: HistogramVec = register_histogram_vec!(
        "http_response_size_bytes",
        "HTTP response size in bytes.",
        &["method", "path"],
        exponential_buckets(100.0, 2.0, 10).unwrap()
    )
    .unwrap();
}

fn exponential_buckets(start: f64, factor: f64, count: usize) -> Result<Vec<f64>, String> {
    if start <= 0.0 || factor <= 1.0 || count == 0 {
        return Err("Invalid bucket parameters".to_string());
    }

    let mut buckets = Vec::with_capacity(count);
    let mut current = start;
    for _ in 0..count {
        buckets.push(current);
        current *= factor;
    }
    Ok(buckets)
}

#[derive(Clone)]
pub struct MetricsMiddleware;

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService { service })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
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
        let method = req.method().to_string();
        let path = req.path().to_string();
        let start_time = Instant::now();
        let request_id = Uuid::new_v4();

        // Record request size
        if let Some(len) = req
            .headers()
            .get("content-length")
            .and_then(|h| h.to_str().ok())
        {
            if let Ok(size) = len.parse::<f64>() {
                HTTP_REQUEST_SIZE_BYTES
                    .with_label_values(&[&method, &path])
                    .observe(size);
            }
        }

        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            "Incoming request"
        );

        let fut = self.service.call(req);
        Box::pin(async move {
            let response = fut.await;
            let duration = start_time.elapsed().as_secs_f64();

            match &response {
                Ok(res) => {
                    let status = res.status().as_u16().to_string();
                    HTTP_REQUESTS_TOTAL
                        .with_label_values(&[&method, &path, &status])
                        .inc();

                    HTTP_REQUEST_DURATION_SECONDS
                        .with_label_values(&[&method, &path])
                        .observe(duration);

                    // Record response size
                    if let Some(len) = res
                        .headers()
                        .get("content-length")
                        .and_then(|h| h.to_str().ok())
                    {
                        if let Ok(size) = len.parse::<f64>() {
                            HTTP_RESPONSE_SIZE_BYTES
                                .with_label_values(&[&method, &path])
                                .observe(size);
                        }
                    }

                    info!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        status = %status,
                        duration = %duration,
                        "Request completed"
                    );
                }
                Err(e) => {
                    error!(
                        request_id = %request_id,
                        method = %method,
                        path = %path,
                        error = %e,
                        duration = %duration,
                        "Request failed"
                    );
                }
            }

            response
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::{web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_metrics_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware::new())
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Verify metrics were recorded
        let metric = HTTP_REQUESTS_TOTAL
            .with_label_values(&["GET", "/test", "200"])
            .get();
        assert_eq!(metric, 1);
    }
}
