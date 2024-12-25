use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error};
use chrono::Utc;
use futures::future::{ok, Ready};
use futures::Future;
use serde_json::Value;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct Logger {
    logs: Arc<Mutex<Vec<Value>>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn log(&self, entry: Value) {
        let mut logs = self.logs.lock().await;
        logs.push(entry);
    }

    pub async fn get_logs(&self) -> Vec<Value> {
        let logs = self.logs.lock().await;
        logs.clone()
    }
}

pub struct LoggerMiddleware {
    logger: Arc<Logger>,
}

impl LoggerMiddleware {
    pub fn new(logger: Arc<Logger>) -> Self {
        Self { logger }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LoggerMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggerMiddlewareService {
            service,
            logger: self.logger.clone(),
        })
    }
}

pub struct LoggerMiddlewareService<S> {
    service: S,
    logger: Arc<Logger>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddlewareService<S>
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
        let logger = self.logger.clone();
        let start_time = Utc::now();
        let request_id = Uuid::new_v4().to_string();

        let fut = self.service.call(req);
        Box::pin(async move {
            let result = fut.await;
            let end_time = Utc::now();
            let duration = end_time.signed_duration_since(start_time).num_milliseconds();

            match &result {
                Ok(res) => {
                    let log_entry = serde_json::json!({
                        "request_id": request_id,
                        "timestamp": end_time,
                        "method": res.request().method().as_str(),
                        "path": res.request().path(),
                        "status": res.status().as_u16(),
                        "duration_ms": duration,
                        "level": "info"
                    });
                    logger.log(log_entry).await;
                }
                Err(e) => {
                    let log_entry = serde_json::json!({
                        "request_id": request_id,
                        "timestamp": end_time,
                        "error": e.to_string(),
                        "level": "error"
                    });
                    logger.log(log_entry).await;
                }
            }

            result
        })
    }
}

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize any logging-specific setup here
    // For example, setting up file handlers, configuring log levels, etc.
    Ok(())
} 