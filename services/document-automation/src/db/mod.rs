use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[allow(dead_code)]
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
} 