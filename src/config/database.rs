use dotenvy::dotenv;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug)]
pub struct DatabaseConfig {
    user: String,
    password: String,
    host: String,
    port: u16,
    database: String,
    max_connections: u32,
    acquire_timeout: Duration,
    ssl_mode: PgSslMode,
    statement_timeout: Duration,
    pool_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            user: env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: env::var("DB_PASSWORD").expect("DB_PASSWORD must be set"),
            host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            database: env::var("DB_NAME").unwrap_or_else(|_| "postgres".to_string()),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            acquire_timeout: Duration::from_secs(
                env::var("DB_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "3".to_string())
                    .parse()
                    .unwrap_or(3),
            ),
            ssl_mode: if env::var("DB_SSL_MODE").unwrap_or_else(|_| "prefer".to_string())
                == "require"
            {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            },
            statement_timeout: Duration::from_secs(
                env::var("DB_STATEMENT_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            ),
            pool_timeout: Duration::from_secs(
                env::var("DB_POOL_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
            ),
        }
    }
}

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let config = DatabaseConfig::default();

    info!("Initializing database connection pool");

    let options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.user)
        .password(&config.password)
        .database(&config.database)
        .ssl_mode(config.ssl_mode)
        // Set statement timeout to prevent long-running queries
        .statement_timeout(config.statement_timeout)
        // Enable application_name for better monitoring
        .application_name("lota_api");

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(Some(Duration::from_secs(180)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .connect_with(options)
        .await?;

    // Verify connection pool
    match pool.acquire().await {
        Ok(_) => info!("Successfully established database connection pool"),
        Err(e) => {
            error!("Failed to verify database connection: {}", e);
            return Err(e);
        }
    }

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Executor;

    #[tokio::test]
    async fn test_database_connection() {
        dotenv().ok();

        match create_pool().await {
            Ok(pool) => {
                // Test connection with a simple query
                match pool.execute("SELECT 1").await {
                    Ok(_) => info!("Successfully connected to database"),
                    Err(e) => panic!("Failed to execute query: {}", e),
                }

                // Test connection timeout
                match pool.execute("SELECT pg_sleep(5)").await {
                    Ok(_) => info!("Long-running query completed"),
                    Err(e) => info!("Expected timeout error: {}", e),
                }
            }
            Err(e) => panic!("Failed to create connection pool: {}", e),
        }
    }
}
