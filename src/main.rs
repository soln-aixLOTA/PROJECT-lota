use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod error;
mod logging;
mod metrics;
mod rate_limit;
mod routes;
mod secrets;
mod tenant;
mod usage_tracking;
mod validation;
mod worker_pool;

use config::AppConfig;
use metrics::init_metrics;
use rate_limit::RateLimitMiddleware;
use routes::{api, health};
use secrets::SecretsManager;
use tenant::TenantMiddleware;
use usage_tracking::UsageTrackingMiddleware;
use validation::ValidationMiddleware;
use worker_pool::AdaptiveWorkerPool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_thread_names(true)
        .with_ansi(true)
        .with_env_filter("info")
        .init();

    // Load configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");
    let host = config.host.clone();
    let port = config.port;

    // Initialize metrics
    init_metrics();

    // Initialize secrets manager
    let secrets_manager = SecretsManager::new(&config).await;

    // Initialize worker pool
    let worker_pool = AdaptiveWorkerPool::new(config.clone()).await;

    info!("Starting API Gateway server on {}:{}", host, port);

    let config = web::Data::new(config);
    let secrets_manager = web::Data::new(secrets_manager);
    let worker_pool = web::Data::new(worker_pool);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(TenantMiddleware::new())
            .wrap(RateLimitMiddleware::new())
            .wrap(ValidationMiddleware::new())
            .wrap(UsageTrackingMiddleware::new())
            .app_data(config.clone())
            .app_data(secrets_manager.clone())
            .app_data(worker_pool.clone())
            .service(health::health_check)
            .service(routes::metrics::get_metrics)
            .service(web::scope("/api/v1").configure(api::config))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
