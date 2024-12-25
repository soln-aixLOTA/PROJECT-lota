use actix_web::{get, HttpResponse};
use serde_json::json;
use chrono;

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
} 