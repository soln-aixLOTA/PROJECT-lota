use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub rate_limit_per_second: u32,
    pub rate_limit_burst: u32,
    pub services: ServicesConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub backlog: i32,
    pub keep_alive: Option<u64>,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub shutdown_timeout: u64,
    pub max_connection_rate: Option<u32>,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServicesConfig {
    pub user_service_url: String,
    pub inference_service_url: String,
    pub attestation_service_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::new()
    }

    pub fn new() -> Result<Self, ConfigError> {
        let builder = ConfigBuilder::builder()
            // Set defaults
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8080)?
            .set_default("server.workers", 4)?
            .set_default("rate_limit_per_second", 100)?
            .set_default("rate_limit_burst", 50)?
            // Load from file
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            // Load from environment
            .add_source(
                Environment::with_prefix("LOTA")
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
