use actix_web::{get, HttpResponse, Responder};
use serde_json::json;
use std::time::SystemTime;

#[get("/health")]
pub async fn health_check() -> impl Responder {
    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    HttpResponse::Ok().json(json!({
        "status": "ok",
        "uptime": uptime,
        "version": env!("CARGO_PKG_VERSION")
    }))
}
