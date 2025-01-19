use actix_web::{web, App, HttpServer};
use lotabots_config::AppConfig;
use lotabots_error::Result;
use tracing::info;

pub mod handlers;
pub mod models;
pub mod repository;
pub mod routes;
pub mod services;

pub async fn run_server(config: AppConfig) -> Result<()> {
    info!("Starting Resource Management service...");

    // Initialize database connection
    let pool = lotabots_db::create_pool(&config.database).await?;

    // Run database migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| lotabots_error::AppError::DatabaseError(e.to_string()))?;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::configure)
    })
    .bind((config.server.host.as_str(), config.server.port))?
    .run()
    .await
    .map_err(|e| e.into())
}
