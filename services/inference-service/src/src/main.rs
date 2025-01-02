use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use lota_inference_service::{
    config::Config, handlers, repositories::InferenceRepository, services::InferenceService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::new().expect("Failed to load configuration");
    let config = web::Data::new(config.clone());

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.url)
        .await
        .expect("Failed to create database pool");

    // Create repositories
    let inference_repository = web::Data::new(InferenceRepository::new(pool));

    // Create services
    let inference_service = web::Data::new(InferenceService::new(config.get_ref().clone()));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(inference_repository.clone())
            .app_data(inference_service.clone())
            .configure(handlers::configure_routes)
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .workers(config.server.workers)
    .run()
    .await
}
