use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> impl Responder {
    // TODO: Add more comprehensive health checks (e.g., database connectivity)
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "attestation",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
