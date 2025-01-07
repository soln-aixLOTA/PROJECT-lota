use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::core::AppState;

pub async fn health_check(data: web::Data<AppState>) -> HttpResponse {
    // Try to query the database
    match sqlx::query("SELECT 1").execute(&data.db).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status": "healthy",
            "database": "connected",
            "storage": "available"
        })),
        Err(e) => HttpResponse::ServiceUnavailable().json(json!({
            "status": "unhealthy",
            "error": format!("Database error: {}", e)
        })),
    }
} 