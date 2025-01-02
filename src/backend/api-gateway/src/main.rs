use std::time::Duration;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::config::Config;
use crate::state::AppState;

mod config;
mod error;
mod routes;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Initialize tracing
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Failed to create tracer");

    let subscriber = Registry::default()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(EnvFilter::from_default_env());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // Initialize application state
    let state = web::Data::new(AppState::new(&config));

    // Start HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .configure(routes::configure)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .workers(config.server.workers as usize)
    .backlog(config.server.backlog.try_into().unwrap())
    .keep_alive(Duration::from_secs(config.server.keep_alive.unwrap_or(75)))
    .client_request_timeout(Duration::from_secs(config.server.client_timeout))
    .client_disconnect_timeout(Duration::from_secs(config.server.client_shutdown))
    .max_connection_rate(config.server.max_connection_rate.unwrap_or(256) as usize)
    .max_connections(config.server.max_connections as usize);

    let result = server.run().await;

    // Shutdown tracing
    global::shutdown_tracer_provider();

    result
}
