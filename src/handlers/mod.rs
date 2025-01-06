use axum::{
    routing::{get, post},
    Router,
};

use crate::core::AppState;

pub mod documents;
pub mod workflows;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/documents", documents::routes())
        .nest("/workflows", workflows::routes())
}
