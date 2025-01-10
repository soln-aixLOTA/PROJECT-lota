use actix_web::{get, web, HttpResponse};
use serde_json::json;
use sqlx::PgPool;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
}

#[get("")]
pub async fn health_check(pool: web::Data<PgPool>) -> HttpResponse {
    let db_healthy = sqlx::query("SELECT 1")
        .fetch_one(pool.get_ref())
        .await
        .is_ok();

    if db_healthy {
        HttpResponse::Ok().json(json!({
            "status": "healthy",
            "services": {
                "database": "up",
            }
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "services": {
                "database": "down",
            }
        }))
    }
} 