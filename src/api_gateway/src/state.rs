use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use prometheus::{Histogram, HistogramOpts, IntCounter, Opts, Registry};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::error::Error;

#[derive(Clone)]
pub struct Metrics {
    pub requests_total: IntCounter,
    pub requests_duration: Histogram,
    pub active_connections: prometheus::Gauge,
    pub rate_limited_requests: IntCounter,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub metrics: Arc<Metrics>,
    pub rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, Error> {
        let config = Arc::new(config);
        let metrics = Arc::new(create_metrics()?);
        let rate_limiter = Arc::new(create_rate_limiter(&config));

        Ok(AppState {
            config,
            metrics,
            rate_limiter,
        })
    }
}

fn create_rate_limiter(config: &Config) -> RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
    let per_second = std::num::NonZeroU32::new(config.rate_limit_per_second)
        .expect("rate_limit_per_second must be non-zero");
    let burst = std::num::NonZeroU32::new(config.rate_limit_burst)
        .expect("rate_limit_burst must be non-zero");

    RateLimiter::direct(Quota::per_second(per_second).allow_burst(burst))
}

fn create_metrics() -> Result<Metrics, Error> {
    let registry = Registry::new();

    let requests_total = IntCounter::new("api_requests_total", "Total number of API requests")
        .map_err(|e| Error::Internal(e.to_string()))?;

    let requests_duration = Histogram::with_opts(HistogramOpts::new(
        "api_request_duration_seconds",
        "API request duration in seconds",
    ))
    .map_err(|e| Error::Internal(e.to_string()))?;

    let active_connections =
        prometheus::Gauge::new("api_active_connections", "Number of active connections")
            .map_err(|e| Error::Internal(e.to_string()))?;

    let rate_limited_requests = IntCounter::new(
        "api_rate_limited_requests_total",
        "Total number of rate limited requests",
    )
    .map_err(|e| Error::Internal(e.to_string()))?;

    registry
        .register(Box::new(requests_total.clone()))
        .map_err(|e| Error::Internal(e.to_string()))?;
    registry
        .register(Box::new(requests_duration.clone()))
        .map_err(|e| Error::Internal(e.to_string()))?;
    registry
        .register(Box::new(active_connections.clone()))
        .map_err(|e| Error::Internal(e.to_string()))?;
    registry
        .register(Box::new(rate_limited_requests.clone()))
        .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(Metrics {
        requests_total,
        requests_duration,
        active_connections,
        rate_limited_requests,
    })
}
