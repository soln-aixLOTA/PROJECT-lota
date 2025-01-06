use anyhow::Result;
use hardware_attestation::{api, storage::StorageClient, HardwareVerifier};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_thread_names(true)
        .with_ansi(true)
        .pretty()
        .init();

    info!("Starting hardware attestation service...");

    // Initialize hardware verifier
    let verifier = HardwareVerifier::new()?;

    // Initialize storage client
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/attestation".to_string());

    let storage = StorageClient::new(&database_url).await?;
    info!("Connected to database");

    // Create the API router
    let app = api::create_router(verifier, storage).layer(TraceLayer::new_for_http());

    // Bind to localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    // Start the server
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
