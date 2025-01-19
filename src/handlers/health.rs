use actix_web::{get, web, HttpResponse};
use serde_json::json;

#[get("")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/health").service(health_check));
}
