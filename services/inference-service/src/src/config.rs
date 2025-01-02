use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub inference: InferenceConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct InferenceConfig {
    pub model_path: String,
    pub max_batch_size: usize,
    pub timeout_ms: u64,
    pub cache_duration_secs: u64,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = ConfigBuilder::builder()
            // Set defaults
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8083)? // Different port from other services
            .set_default("server.workers", 4)?
            .set_default("database.min_connections", 5)?
            .set_default("database.max_connections", 100)?
            .set_default("inference.max_batch_size", 32)?
            .set_default("inference.timeout_ms", 30000)?
            .set_default("inference.cache_duration_secs", 3600)?
            // Load from file
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            // Load from environment
            .add_source(
                Environment::with_prefix("LOTA_INFERENCE")
                    .separator("_")
                    .try_parsing(true),
            );

        let mut config: Config = builder.build()?.try_deserialize()?;

        // Update workers count based on CPU cores if not set in config
        if config.server.workers == 4 {
            config.server.workers = num_cpus::get();
        }

        Ok(config)
    }
}
