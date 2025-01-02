use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pool: Arc<Pool<Postgres>>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&*self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/test_db".to_string());

        let db = Database::new(&database_url).await;
        assert!(db.is_ok());
    }
} 