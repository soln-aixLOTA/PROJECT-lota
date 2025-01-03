use actix_web::{web, App, HttpServer};
use common::{init as common_init, LogContext};
use tracing::info;

mod models;
mod handlers;
mod db;
mod config;
mod error;

use config::Config;
use db::Database;
use error::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    // Initialize common components
    common_init().await?;

    // Initialize logging context
    let log_ctx = LogContext::new("user-management", "development");
    info!(
        service = %log_ctx.service,
        environment = %log_ctx.environment,
        "Starting user management service"
    );

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded successfully");

    // Initialize database
    let database = Database::new(&config.database_url).await?;
    info!("Database connection established");

    // Run database migrations
    database.run_migrations().await?;
    info!("Database migrations completed");

    // Create application state
    let app_state = web::Data::new(AppState {
        config: config.clone(),
        database: database.clone(),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(actix_web::middleware::Logger::default())
            .service(
                web::scope("/api/v1")
                    .configure(handlers::users::configure)
                    .configure(handlers::tenants::configure)
                    .configure(handlers::auth::configure),
            )
    })
    .bind((config.host, config.port))?
    .run()
    .await?;

    Ok(())
}

pub struct AppState {
    config: Config,
    database: Database,
} 