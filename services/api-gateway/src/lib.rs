use actix_web::{web, App, HttpServer};
use lotabots_config::AppConfig;
use lotabots_error::Result;
use tracing::info;

pub mod auth;
pub mod handlers;
pub mod middleware;
pub mod routes;

/// Run the API Gateway server
pub async fn run_server(config: AppConfig) -> Result<()> {
    info!("Starting API Gateway server...");

    let server = HttpServer::new(move || {
        App::new()
            // Configure logging middleware
            .wrap(middleware::logging::RequestLogger::new())
            // Configure metrics middleware
            .wrap(middleware::metrics::MetricsCollector::new())
            // Configure authentication middleware
            .wrap(middleware::auth::AuthenticationMiddleware::new())
            // Configure rate limiting middleware
            .wrap(middleware::rate_limit::RateLimiter::new())
            // Configure routes
            .configure(routes::configure)
            // Configure error handlers
            .app_data(web::JsonConfig::default().error_handler(|err, _| {
                handlers::error::handle_json_error(err).into()
            }))
    })
    .bind((config.api.host, config.api.port))?
    .run();

    info!("API Gateway listening on {}:{}", config.api.host, config.api.port);

    server.await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lotabots_testing::test_utils;

    #[tokio::test]
    async fn test_server_startup() {
        let config = test_utils::get_test_config();
        assert!(run_server(config).await.is_ok());
    }
}
