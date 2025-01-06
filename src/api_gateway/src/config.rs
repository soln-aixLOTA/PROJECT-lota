use config::{Config, ConfigError, Environment, File};
use lazy_static::lazy_static;
use lotabots_common::logging::LoggingConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiry_hours: u32,
    pub refresh_token_expiry_days: u32,
    pub password_hash_iterations: u32,
    pub minimum_password_length: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub free_tier_rpm: u32,
    pub pro_tier_rpm: u32,
    pub enterprise_tier_rpm: u32,
    pub burst_multiplier: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MetricsConfig {
    pub prometheus_endpoint: String,
    pub metrics_prefix: String,
    pub collection_interval_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Server settings
    pub host: String,
    pub port: u16,

    // Worker pool settings
    pub min_workers: usize,
    pub max_workers: usize,
    pub worker_queue_size: usize,
    pub max_concurrent_requests: usize,

    // Rate limiting
    pub rate_limit_requests_per_second: u32,

    // Adaptive scaling settings
    pub scaling_check_interval: Duration,
    pub scale_up_threshold: f64,   // Load factor above which to scale up
    pub scale_down_threshold: f64, // Load factor below which to scale down
    pub scale_up_factor: f64,      // Factor by which to increase workers
    pub scale_down_factor: f64,    // Factor by which to decrease workers
    pub min_scale_interval: Duration, // Minimum time between scaling operations

    // Resource thresholds
    pub max_cpu_usage: f64,    // Maximum CPU usage percentage
    pub max_memory_usage: f64, // Maximum memory usage percentage
    pub max_gpu_usage: f64,    // Maximum GPU usage percentage

    // Timeouts
    pub request_timeout: Duration,
    pub worker_shutdown_timeout: Duration,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            min_workers: 5,
            max_workers: 50,
            worker_queue_size: 100,
            max_concurrent_requests: 1000,
            rate_limit_requests_per_second: 1000,
            scaling_check_interval: Duration::from_secs(30),
            scale_up_threshold: 0.75,
            scale_down_threshold: 0.25,
            scale_up_factor: 1.5,
            scale_down_factor: 0.75,
            min_scale_interval: Duration::from_secs(60),
            max_cpu_usage: 80.0,
            max_memory_usage: 80.0,
            max_gpu_usage: 80.0,
            request_timeout: Duration::from_secs(30),
            worker_shutdown_timeout: Duration::from_secs(10),
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut s = Config::builder();

        // 1. repo_tools.toml (if applicable and found)
        s.add_source(File::with_name("repo_tools").required(false)); // Assuming api-gateway's repo_tools.toml

        // 2. api-gateway specific defaults (can be in a default.toml or similar)
        s.add_source(File::with_name("config/default").required(false));

        // 3. repo.toml (if found in the project root)
        if let Ok(repo_root) = env::var("REPO_ROOT") {
            s.add_source(File::with_name(format!("{}/repo", repo_root)).required(false));
        }

        // 4. ~/.nvidia-omniverse/config/global_repo.toml
        if let Some(home_dir) = env::home_dir() {
            s.add_source(
                File::with_name(format!(
                    "{}/.nvidia-omniverse/config/global_repo",
                    home_dir.display()
                ))
                .required(false),
            );
        }

        // 5. user.repo.toml (if found in the project root)
        if let Ok(repo_root) = env::var("REPO_ROOT") {
            s.add_source(File::with_name(format!("{}/user.repo", repo_root)).required(false));
        }

        // 6. Environment variables (with a prefix, e.g., API_)
        s.add_source(Environment::with_prefix("API"));

        let config = s.build()?;

        // Convert to AppConfig struct
        config.try_deserialize()
    }

    fn validate(&self) -> Result<(), config::ConfigError> {
        if self.min_workers > self.max_workers {
            return Err(config::ConfigError::Message(
                "min_workers cannot be greater than max_workers".to_string(),
            ));
        }

        if self.scale_up_threshold <= self.scale_down_threshold {
            return Err(config::ConfigError::Message(
                "scale_up_threshold must be greater than scale_down_threshold".to_string(),
            ));
        }

        if self.scale_up_factor <= 1.0 {
            return Err(config::ConfigError::Message(
                "scale_up_factor must be greater than 1.0".to_string(),
            ));
        }

        if self.scale_down_factor >= 1.0 {
            return Err(config::ConfigError::Message(
                "scale_down_factor must be less than 1.0".to_string(),
            ));
        }

        Ok(())
    }

    // Helper methods for common calculations
    pub fn get_max_queue_size(&self) -> usize {
        self.worker_queue_size * self.max_workers
    }

    pub fn get_target_workers(&self, load_factor: f64) -> usize {
        let target = (self.min_workers as f64 * load_factor).ceil() as usize;
        target.clamp(self.min_workers, self.max_workers)
    }

    pub fn should_scale_up(&self, load_factor: f64) -> bool {
        load_factor >= self.scale_up_threshold
    }

    pub fn should_scale_down(&self, load_factor: f64) -> bool {
        load_factor <= self.scale_down_threshold
    }
}

// Global configuration holder
lazy_static! {
    static ref CONFIG: Arc<RwLock<AppConfig>> = Arc::new(RwLock::new(AppConfig::from_env()));
}

// Helper functions to access config
pub async fn get_config() -> AppConfig {
    CONFIG.read().await.clone()
}

pub async fn update_config(new_config: AppConfig) {
    let mut config = CONFIG.write().await;
    *config = new_config;
}

// Validation functions
impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        // Validate database configuration
        if self.database.max_connections == 0 {
            return Err("Database max_connections must be greater than 0".into());
        }
        if self.database.url.is_empty() {
            return Err("Database URL cannot be empty".into());
        }

        // Validate security configuration
        if self.security.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters long".into());
        }
        if self.security.minimum_password_length < 8 {
            return Err("Minimum password length must be at least 8 characters".into());
        }

        // Validate rate limiting configuration
        if self.rate_limit.burst_multiplier < 1.0 {
            return Err("Rate limit burst multiplier must be greater than 1.0".into());
        }

        // Validate metrics configuration
        if self.metrics.collection_interval_seconds == 0 {
            return Err("Metrics collection interval must be greater than 0".into());
        }

        Ok(())
    }
}

// Configuration update events
#[derive(Debug, Clone)]
pub struct ConfigUpdateEvent {
    pub old_config: AppConfig,
    pub new_config: AppConfig,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Configuration change listener
pub async fn watch_config_changes<F>(mut callback: F)
where
    F: FnMut(ConfigUpdateEvent) + Send + 'static,
{
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    let mut last_config = get_config().await;

    loop {
        interval.tick().await;

        match AppConfig::new() {
            Ok(new_config) => {
                if new_config != last_config {
                    let event = ConfigUpdateEvent {
                        old_config: last_config.clone(),
                        new_config: new_config.clone(),
                        timestamp: chrono::Utc::now(),
                    };

                    // Update the global config
                    update_config(new_config.clone()).await;

                    // Notify the callback
                    callback(event);

                    last_config = new_config;
                }
            }
            Err(e) => {
                tracing::error!("Failed to reload configuration: {}", e);
            }
        }
    }
}

// Initialize logging configuration
pub fn initialize_logging() {
    let config = LoggingConfig {
        service_name: "api-gateway".to_string(),
        environment: "production".to_string(),
        log_level: tracing::Level::INFO,
        json_format: true,
    };
    config.init();
}
