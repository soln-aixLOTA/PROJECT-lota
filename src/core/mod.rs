use sqlx::PgPool;
use std::sync::Arc;

use crate::storage::StorageProvider;

pub mod error;

pub struct AppState {
    pub db: PgPool,
    pub storage: Arc<dyn StorageProvider>,
}

impl AppState {
    pub fn new(db: PgPool, storage: Arc<dyn StorageProvider>) -> Self {
        Self { db, storage }
    }
}
