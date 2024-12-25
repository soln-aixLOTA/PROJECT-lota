use serde::Deserialize;
use std::time::Duration;
use config::{Config as ConfigSource, Environment, File};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    // Server configuration
    pub port: u16,
    pub workers: usize,
    pub backlog: i32,
    pub keep_alive: Duration,
    pub client_timeout: Duration,
    pub client_shutdown: Duration,
    pub shutdown_timeout: Duration,
    pub max_connection_rate: Option<usize>,
    pub max_connections: usize,
    
    // Service endpoints
    pub user_service_url: String,
    pub inference_service_url: String,
    pub training_service_url: String,
    
    // Authentication
    pub auth_enabled: bool,
    pub jwt_public_key: Option<String>,
    
    // Rate limiting
    pub rate_limit_enabled: bool,
    pub rate_limit_per_second: u32,
    pub rate_limit_burst: u32,
    
    // Tracing
    pub jaeger_endpoint: Option<String>,
    
    // Metrics
    pub metrics_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut builder = ConfigSource::builder()
            // Start with default values
            .set_default("port", 8080)?
            .set_default("workers", num_cpus::get())?
            .set_default("backlog", 2048)?
            .set_default("keep_alive", 75)?  // seconds
            .set_default("client_timeout", 60)?  // seconds
            .set_default("client_shutdown", 30)?  // seconds
            .set_default("shutdown_timeout", 30)?  // seconds
            .set_default("max_connections", 25_000)?
            .set_default("auth_enabled", true)?
            .set_default("rate_limit_enabled", true)?
            .set_default("rate_limit_per_second", 1000)?
            .set_default("rate_limit_burst", 50)?
            .set_default("metrics_port", 9090)?;
            
        // Layer on the environment-specific values
        if let Ok(env) = std::env::var("RUN_MODE") {
            builder = builder.add_source(File::with_name(&format!("config/{}", env)).required(false));
        }
        
        // Add environment variables
        builder = builder.add_source(Environment::with_prefix("API_GATEWAY"));
        
        // Build the config
        let config = builder.build()?;
        
        // Convert into our Config struct
        let mut config: Config = config.try_deserialize()?;
        
        // Convert durations from seconds to Duration
        config.keep_alive = Duration::from_secs(config.keep_alive.as_secs());
        config.client_timeout = Duration::from_secs(config.client_timeout.as_secs());
        config.client_shutdown = Duration::from_secs(config.client_shutdown.as_secs());
        config.shutdown_timeout = Duration::from_secs(config.shutdown_timeout.as_secs());
        
        Ok(config)
    }
} 