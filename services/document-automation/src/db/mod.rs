mod documents;
mod workflows;

pub use documents::DocumentRepository;
pub use workflows::WorkflowRepository;

use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::core::{DatabaseConfig, DocumentResult};

pub async fn create_pool(config: &DatabaseConfig) -> DocumentResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_seconds))
        .connect(&config.url)
        .await
        .map_err(|e| crate::core::DocumentError::DatabaseError(e.to_string()))?;

    Ok(pool)
}
