use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserQuery {
    pub id: String,
}

pub async fn get_user(query: web::Query<UserQuery>) -> HttpResponse {
    // User retrieval logic
    HttpResponse::Ok().json(json!({"id": query.id, "name": "Test User"}))
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/users").route(web::get().to(get_user)));
} 