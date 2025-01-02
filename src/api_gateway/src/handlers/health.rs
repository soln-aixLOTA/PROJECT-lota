use actix_web::{get, web, HttpResponse};
use serde_json::json;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

#[get("/health")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
