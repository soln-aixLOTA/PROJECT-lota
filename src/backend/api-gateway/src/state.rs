use std::sync::Arc;

use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use nonzero_ext::nonzero;
use prometheus::{Histogram, HistogramOpts, IntCounter, Registry};
use reqwest::Client;

use crate::config::Config;

pub struct AppState {
    pub config: Arc<Config>,
    pub http_client: Client,
    pub request_counter: IntCounter,
    pub request_duration: Histogram,
    pub rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
    pub cache: DashMap<String, Vec<u8>>,
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        // Initialize metrics
        let registry = Registry::new();
        let request_counter =
            IntCounter::new("api_requests_total", "Total number of API requests").unwrap();
        let request_duration = Histogram::with_opts(
            HistogramOpts::new(
                "api_request_duration_seconds",
                "API request duration in seconds",
            )
            .buckets(vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
            ]),
        )
        .unwrap();

        registry
            .register(Box::new(request_counter.clone()))
            .unwrap();
        registry
            .register(Box::new(request_duration.clone()))
            .unwrap();

        // Initialize rate limiter with default values
        let rate_limiter = RateLimiter::direct(Quota::per_second(nonzero!(100u32)));

        Self {
            config: Arc::new(config.clone()),
            http_client: Client::new(),
            request_counter,
            request_duration,
            rate_limiter: Arc::new(rate_limiter),
            cache: DashMap::new(),
        }
    }
}
