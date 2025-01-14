use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, Ready};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use crate::telemetry::metrics::ApiMetrics;

pub struct MetricsMiddleware;

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
        ok(MetricsMiddlewareService { service })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
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
        let metrics = ApiMetrics::global();
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        // Record the request
        metrics.record_request(&method, &path);
        metrics.connection_change(1);

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await;
            let duration = start_time.elapsed().as_secs_f64();

            match &res {
                Ok(response) => {
                    let status = response.status();
                    if status.is_client_error() || status.is_server_error() {
                        metrics.record_error(
                            if status.is_client_error() {
                                "client_error"
                            } else {
                                "server_error"
                            },
                            status.as_u16(),
                        );
                    }
                    if status == 401 || status == 403 {
                        metrics.record_auth_failure(if status == 401 {
                            "unauthorized"
                        } else {
                            "forbidden"
                        });
                    }
                }
                Err(e) => {
                    metrics.record_error("internal_error", 500);
                    log::error!("Request error: {:?}", e);
                }
            }

            metrics.record_request_duration(duration, &path);
            metrics.connection_change(-1);
            res
        })
    }
}

/// Endpoint to expose metrics in Prometheus format
pub async fn metrics_handler() -> HttpResponse {
    use crate::telemetry::metrics::get_metrics;
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(get_metrics())
}
