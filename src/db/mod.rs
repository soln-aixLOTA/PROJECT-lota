use sqlx::PgPool;
use crate::error::AppError;
pub type DbResult<T> = Result<T, AppError>;

pub mod documents;
pub mod users;
pub mod workflows;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

pub async fn init_db(database_url: &str) -> DbResult<PgPool> {
    PgPool::connect(database_url)
        .await
        .map_err(|e| AppError::Database(e))
}
