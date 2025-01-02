use sqlx::PgPool;
use std::sync::Arc;

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod utils;

#[derive(Clone)]
pub struct AppState {
    pub pool: Arc<PgPool>,
    pub config: Arc<config::Config>,
}

impl AppState {
    pub fn new(pool: PgPool, config: config::Config) -> Self {
        Self {
            pool: Arc::new(pool),
            config: Arc::new(config),
        }
    }
}
