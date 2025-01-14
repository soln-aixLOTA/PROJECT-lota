use actix_web::web;

pub mod auth;
pub mod documents;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/api")
                .configure(auth::config)
                .configure(documents::config)
        )
        .route("/health", web::get().to(health));
}

async fn health() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok()
        .json(serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
}
