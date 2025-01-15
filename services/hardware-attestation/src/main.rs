use actix_web::{web, App, HttpServer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting hardware attestation service");

    // Initialize NVML
    let nvml = hardware_attestation::NvmlWrapper::new()
        .expect("Failed to initialize NVML");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(nvml.clone()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
