use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod auth;
mod core;
mod db;
mod models;
mod storage;

use crate::{
    api::{create_workflow, delete_workflow, get_workflow, list_workflows},
    core::AppState,
    storage::init_storage,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database connection
    let db = sqlx::postgres::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    // Initialize storage backend
    let storage = init_storage(
        &std::env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string()),
        &std::env::var("STORAGE_CONFIG").unwrap_or_else(|_| "./data".to_string()),
    )
    .await?;

    // Create application state
    let state = AppState::new(db, storage);

    // Build our application with some routes
    let app = Router::new()
        .route("/workflows", post(create_workflow))
        .route("/workflows", get(list_workflows))
        .route("/workflows/:id", get(get_workflow))
        .route("/workflows/:id", delete(delete_workflow))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;

    Ok(())
}
