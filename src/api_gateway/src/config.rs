use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub services: ServicesConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: i32,
    pub backlog: i32,
    pub keep_alive: Option<u64>,
    pub client_timeout: u64,
    pub client_shutdown: u64,
    pub max_connection_rate: Option<u32>,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServicesConfig {
    pub attestation_url: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            // Set defaults
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 8080)?
            .set_default("server.workers", num_cpus::get() as i32)?
            .set_default("server.backlog", 1024)?
            .set_default("server.keep_alive", Some(75u64))?
            .set_default("server.client_timeout", 60u64)?
            .set_default("server.client_shutdown", 30u64)?
            .set_default("server.max_connection_rate", Some(256u32))?
            .set_default("server.max_connections", 25_000u32)?
            .set_default("services.attestation_url", "http://attestation:8080")?
            // Add config file if it exists
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP_"
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        config.try_deserialize()
    }
}
