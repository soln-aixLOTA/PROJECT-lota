use actix_web::{get, HttpResponse};
use crate::models::ApiResponse;

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success("OK"))
}
