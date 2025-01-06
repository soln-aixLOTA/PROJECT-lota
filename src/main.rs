use actix_web::{App, HttpServer};
use axum::{
    routing::{get, post},
    Router,
};
use handlers::configure_routes;
use logging::init_logger;
use sqlx::PgPool;
use std::sync::Arc;
use storage::{LocalStorage, StorageProvider};

mod errors;
mod handlers;
mod logging;
mod middleware;
mod resource_management;

pub struct AppState {
    db: PgPool,
    storage: StorageProvider,
}

impl AppState {
    pub fn new(db: PgPool, storage: StorageProvider) -> Self {
        Self { db, storage }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let db = PgPool::connect("postgres://postgres:postgres@localhost/document_automation").await?;
    let storage = StorageProvider::Local(LocalStorage::new("./storage"));

    let state = Arc::new(AppState::new(db, storage));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
