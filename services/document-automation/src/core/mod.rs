pub mod error;

use std::fmt::Debug;
use std::sync::Arc;

use sqlx::PgPool;

use crate::storage::StorageBackend;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub storage: Arc<Box<dyn StorageBackend>>,
}

impl AppState {
    pub fn new(db: PgPool, storage: Box<dyn StorageBackend>) -> Self {
        Self {
            db,
            storage: Arc::new(storage),
        }
    }
}
