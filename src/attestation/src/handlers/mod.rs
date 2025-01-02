use actix_web::{
    web::{self, Data, Json, Path},
    HttpResponse, Responder,
};
use uuid::Uuid;

use crate::{
    errors::Error,
    models::{Attestation, AttestationRequest},
    services::{attestation::AttestationService, verification::VerificationService},
    AppState,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/attestations")
            .route("", web::post().to(create_attestation))
            .route("/{id}", web::get().to(get_attestation))
            .route("/verify/{id}", web::post().to(verify_attestation))
            .route("", web::get().to(list_attestations)),
    );
}

async fn create_attestation(
    state: Data<AppState>,
    req: Json<AttestationRequest>,
) -> Result<impl Responder, Error> {
    let service = AttestationService::new(state.pool.clone());
    let attestation = service.create_attestation(req.into_inner()).await?;
    Ok(HttpResponse::Created().json(attestation))
}

async fn get_attestation(state: Data<AppState>, id: Path<Uuid>) -> Result<impl Responder, Error> {
    let service = AttestationService::new(state.pool.clone());
    let attestation = service.get_attestation(id.into_inner()).await?;
    match attestation {
        Some(attestation) => Ok(HttpResponse::Ok().json(attestation)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

async fn verify_attestation(
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, Error> {
    let service = VerificationService::new(state.pool.clone());
    let verified = service.verify_attestation(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(verified))
}

async fn list_attestations(
    state: Data<AppState>,
    query: web::Query<ListAttestationsQuery>,
) -> Result<impl Responder, Error> {
    let service = AttestationService::new(state.pool.clone());
    let attestations = service
        .list_attestations(
            query.model_id,
            query.version_id,
            query.status.clone(),
            query.limit.unwrap_or(10),
            query.offset.unwrap_or(0),
        )
        .await?;
    Ok(HttpResponse::Ok().json(attestations))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListAttestationsQuery {
    pub model_id: Option<Uuid>,
    pub version_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
