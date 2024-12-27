use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;
use std::time::Duration;

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = format!(
        "postgresql://{}:{}@{}:{}/{}",
        env::var("DB_USER").unwrap_or_else(|_| "postgre".to_string()),
        env::var("DB_PASSWORD").unwrap_or_else(|_| "Lhl980107".to_string()),
        env::var("DB_HOST").unwrap_or_else(|_| "10.87.224.2".to_string()),
        env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()),
        env::var("DB_NAME").unwrap_or_else(|_| "postgres".to_string()),
    );

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[tokio::test]
    async fn test_database_connection() {
        dotenv().ok();

        match create_pool().await {
            Ok(pool) => {
                // Try to execute a simple query
                match sqlx::query("SELECT 1").execute(&pool).await {
                    Ok(_) => println!("Successfully connected to database!"),
                    Err(e) => panic!("Failed to execute query: {}", e),
                }
            }
            Err(e) => panic!("Failed to create connection pool: {}", e),
        }
    }
}
