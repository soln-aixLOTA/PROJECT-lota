mod middleware;
mod telemetry;

use actix_web::{web, App, HttpServer};
use middleware::metrics::MetricsMiddleware;
use telemetry::metrics::ApiMetrics;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Initialize metrics
    ApiMetrics::init().expect("Failed to initialize metrics");

    // Start HTTP server
    HttpServer::new(|| {
        App::new().wrap(MetricsMiddleware).service(
            web::scope("/metrics").route("", web::get().to(middleware::metrics::metrics_handler)),
        )
        // Add your other routes and services here
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
