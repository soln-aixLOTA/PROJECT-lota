use crate::config::Config;
use crate::services::{inference::InferenceClient, training::TrainingClient, user::UserClient};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use governor::{RateLimiter, Quota, clock::DefaultClock};
use nonzero_ext::nonzero;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub user_client: Arc<UserClient>,
    pub inference_client: Arc<InferenceClient>,
    pub training_client: Arc<TrainingClient>,
    pub rate_limiters: Arc<DashMap<String, Arc<RateLimiter<String, DefaultClock>>>>,
    pub metrics: Arc<Metrics>,
}

pub struct Metrics {
    pub requests_total: prometheus::IntCounter,
    pub requests_duration: prometheus::Histogram,
    pub active_connections: prometheus::Gauge,
    pub rate_limited_requests: prometheus::IntCounter,
}

impl AppState {
    pub async fn new(config: Config) -> std::io::Result<Self> {
        // Initialize service clients
        let user_client = Arc::new(UserClient::new(&config.user_service_url));
        let inference_client = Arc::new(InferenceClient::new(&config.inference_service_url));
        let training_client = Arc::new(TrainingClient::new(&config.training_service_url));
        
        // Initialize rate limiters map
        let rate_limiters = Arc::new(DashMap::new());
        
        // Initialize metrics
        let metrics = Arc::new(Metrics::new()?);
        
        Ok(Self {
            config,
            user_client,
            inference_client,
            training_client,
            rate_limiters,
            metrics,
        })
    }
    
    pub fn get_rate_limiter(&self, key: &str) -> Arc<RateLimiter<String, DefaultClock>> {
        self.rate_limiters
            .entry(key.to_string())
            .or_insert_with(|| {
                Arc::new(RateLimiter::keyed(
                    Quota::per_second(nonzero!(self.config.rate_limit_per_second.into()))
                        .allow_burst(nonzero!(self.config.rate_limit_burst.into()))
                ))
            })
            .clone()
    }
}

impl Metrics {
    pub fn new() -> std::io::Result<Self> {
        let registry = prometheus::Registry::new();
        
        let requests_total = prometheus::IntCounter::new(
            "api_gateway_requests_total",
            "Total number of requests processed"
        )?;
        
        let requests_duration = prometheus::Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "api_gateway_request_duration_seconds",
                "Request duration in seconds"
            )
            .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        )?;
        
        let active_connections = prometheus::Gauge::new(
            "api_gateway_active_connections",
            "Number of active connections"
        )?;
        
        let rate_limited_requests = prometheus::IntCounter::new(
            "api_gateway_rate_limited_requests_total",
            "Total number of rate-limited requests"
        )?;
        
        registry.register(Box::new(requests_total.clone()))?;
        registry.register(Box::new(requests_duration.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(rate_limited_requests.clone()))?;
        
        Ok(Self {
            requests_total,
            requests_duration,
            active_connections,
            rate_limited_requests,
        })
    }
} 