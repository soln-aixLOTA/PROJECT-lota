use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::core::error::{DocumentError, DocumentResult};
use crate::core::DatabaseConfig;

pub mod documents;
pub mod workflows;

pub use documents::DocumentRepository;
pub use workflows::WorkflowRepository;

pub async fn init_db(config: &DatabaseConfig) -> DocumentResult<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.connection_string())
        .await
        .map_err(|e| DocumentError::DatabaseError(e.to_string()))
}
