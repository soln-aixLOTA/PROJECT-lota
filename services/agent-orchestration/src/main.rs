mod api;
mod core;
mod models;
mod storage;

use axum::{
    routing::{get, post, put},
    Router,
};
use core::AgentManager;
use opentelemetry::sdk::Resource;
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Registry, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    nats: async_nats::Client,
    registry: Registry,
    agent_manager: std::sync::Arc<AgentManager>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize metrics
    let registry = Registry::new_custom(
        Some("agent_orchestration".into()),
        Some(
            [(
                "service".to_string(),
                "Agent Orchestration Service".to_string(),
            )]
            .into(),
        ),
    )?;

    // Connect to NATS
    let nats = async_nats::connect("nats://localhost:4222").await?;

    // Create agent manager
    let agent_manager = std::sync::Arc::new(AgentManager::new(nats.clone()));

    // Create app state
    let state = AppState {
        nats: nats.clone(),
        registry: registry.clone(),
        agent_manager,
    };

    // Create router
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics_handler))
        .route("/agents", post(api::agents::register_agent))
        .route("/agents/:id", get(api::agents::get_agent))
        .route("/agents/:id/tasks", post(api::agents::assign_task))
        .route("/tasks", get(api::tasks::list_tasks))
        .route("/tasks/:id", get(api::tasks::get_task))
        .route("/tasks/:id/status", put(api::tasks::update_task_status))
        .with_state(state);

    // Run server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    tracing::info!("Agent orchestration service listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = state.registry.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        [(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        buffer,
    )
}
