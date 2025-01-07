use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod core;
mod db;
mod handlers;
mod middleware;
mod models;
mod storage;

use crate::core::AppState;
use crate::handlers::configure_routes;
use crate::storage::LocalStorage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Initialize storage
    let storage = Arc::new(LocalStorage::new("./storage"));

    // Create app state
    let state = web::Data::new(AppState::new(pool, storage));

    // Start server
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(configure_routes)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
