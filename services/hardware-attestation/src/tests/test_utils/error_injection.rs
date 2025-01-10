use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Mutex;
use tower::{Layer, Service};

/// Error injection configuration
#[derive(Clone, Debug)]
pub struct ErrorConfig {
    pub error_rate: f64,
    pub latency: Option<Duration>,
    pub timeout: Option<Duration>,
    pub connection_reset: bool,
}

impl Default for ErrorConfig {
    fn default() -> Self {
        Self {
            error_rate: 0.0,
            latency: None,
            timeout: None,
            connection_reset: false,
        }
    }
}

/// Error injection service
#[derive(Clone)]
pub struct ErrorInjector<S> {
    inner: S,
    config: Arc<Mutex<ErrorConfig>>,
}

impl<S> ErrorInjector<S> {
    pub fn new(inner: S, config: ErrorConfig) -> Self {
        Self {
            inner,
            config: Arc::new(Mutex::new(config)),
        }
    }

    pub async fn update_config(&self, config: ErrorConfig) {
        *self.config.lock().await = config;
    }
}

impl<S, Request> Service<Request> for ErrorInjector<S>
where
    S: Service<Request>,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let config = self.config.clone();
        let future = self.inner.call(request);

        Box::pin(async move {
            let config = config.lock().await;

            // Inject random errors based on error rate
            if rand::random::<f64>() < config.error_rate {
                return Err("Injected error".into());
            }

            // Inject latency if configured
            if let Some(latency) = config.latency {
                tokio::time::sleep(latency).await;
            }

            // Simulate connection reset
            if config.connection_reset {
                return Err("Connection reset by peer".into());
            }

            // Apply timeout if configured
            if let Some(timeout) = config.timeout {
                tokio::select! {
                    result = future => result.map_err(Into::into),
                    _ = tokio::time::sleep(timeout) => {
                        Err("Request timed out".into())
                    }
                }
            } else {
                future.await.map_err(Into::into)
            }
        })
    }
}

/// Layer for adding error injection
#[derive(Clone)]
pub struct ErrorInjectionLayer {
    config: ErrorConfig,
}

impl ErrorInjectionLayer {
    pub fn new(config: ErrorConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for ErrorInjectionLayer {
    type Service = ErrorInjector<S>;

    fn layer(&self, service: S) -> Self::Service {
        ErrorInjector::new(service, self.config.clone())
    }
}

/// Helper functions for common error scenarios
pub mod scenarios {
    use super::*;
    use std::time::Duration;

    pub fn network_latency(latency: Duration) -> ErrorConfig {
        ErrorConfig {
            latency: Some(latency),
            ..Default::default()
        }
    }

    pub fn timeout(duration: Duration) -> ErrorConfig {
        ErrorConfig {
            timeout: Some(duration),
            ..Default::default()
        }
    }

    pub fn connection_reset() -> ErrorConfig {
        ErrorConfig {
            connection_reset: true,
            ..Default::default()
        }
    }

    pub fn random_errors(rate: f64) -> ErrorConfig {
        ErrorConfig {
            error_rate: rate,
            ..Default::default()
        }
    }

    pub fn combined_scenario() -> ErrorConfig {
        ErrorConfig {
            error_rate: 0.1,
            latency: Some(Duration::from_millis(100)),
            timeout: Some(Duration::from_secs(5)),
            connection_reset: false,
        }
    }
}
