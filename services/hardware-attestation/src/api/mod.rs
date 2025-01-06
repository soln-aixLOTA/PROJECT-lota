use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    storage::StorageClient, AttestationError, ErrorResponse, HardwareInfo, HardwareVerifier,
};

pub struct ApiState {
    verifier: Arc<HardwareVerifier>,
    storage: Arc<StorageClient>,
}

pub async fn attest_hardware(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<HardwareInfo>, AttestationError> {
    info!("Received hardware attestation request");

    let hardware_info = state.verifier.verify_hardware().await.map_err(|e| {
        error!("Hardware verification failed: {:?}", e);
        e
    })?;

    // Store the attestation result
    state
        .storage
        .store_attestation(&hardware_info)
        .await
        .map_err(|e| {
            error!("Failed to store attestation: {:?}", e);
            e
        })?;

    Ok(Json(hardware_info))
}

pub async fn get_attestation(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<HardwareInfo>, AttestationError> {
    info!("Received request for attestation {}", id);

    let hardware_info = state.storage.get_attestation(id).await.map_err(|e| {
        error!("Failed to retrieve attestation {}: {:?}", id, e);
        e
    })?;

    Ok(Json(hardware_info))
}

pub fn create_router(verifier: HardwareVerifier, storage: StorageClient) -> Router {
    let state = Arc::new(ApiState {
        verifier: Arc::new(verifier),
        storage: Arc::new(storage),
    });

    Router::new()
        .route("/attest", post(attest_hardware))
        .route("/attestation/:id", get(get_attestation))
        .with_state(state)
}
