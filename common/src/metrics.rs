use prometheus::{Registry, Counter, Histogram, IntGauge};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    
    pub static ref REQUEST_COUNTER: Counter = Counter::new(
        "lotabots_requests_total",
        "Total number of requests processed"
    ).unwrap();
    
    pub static ref REQUEST_DURATION: Histogram = Histogram::with_opts(
        histogram_opts!(
            "lotabots_request_duration_seconds",
            "Request duration in seconds"
        )
    ).unwrap();
    
    pub static ref ACTIVE_CONNECTIONS: IntGauge = IntGauge::new(
        "lotabots_active_connections",
        "Number of active connections"
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(REQUEST_COUNTER.clone())).unwrap();
    REGISTRY.register(Box::new(REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_CONNECTIONS.clone())).unwrap();
}
