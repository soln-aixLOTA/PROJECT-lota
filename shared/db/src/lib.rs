use sqlx::postgres::{PgPool, PgPoolOptions};
use thiserror::Error;

pub mod config;
pub mod connection;
pub mod migrations;
pub mod models;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    MigrationError(String),
    #[error("Query error: {0}")]
    QueryError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Database configuration options
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub max_lifetime_secs: u64,
    pub idle_timeout_secs: u64,
}

/// Creates a new database connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_secs))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
        .connect(&config.url)
        .await
        .map_err(DatabaseError::ConnectionError)
}
