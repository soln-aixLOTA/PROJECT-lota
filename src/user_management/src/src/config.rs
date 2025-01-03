use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
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

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = ConfigBuilder::builder()
            // Set defaults
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8081)? // Different port from API Gateway
            .set_default("server.workers", 4)?
            .set_default("database.min_connections", 5)?
            .set_default("database.max_connections", 100)?
            // Load from file
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            // Load from environment
            .add_source(
                Environment::with_prefix("LOTA_USER")
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
