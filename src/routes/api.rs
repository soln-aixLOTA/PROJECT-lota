use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BotConfig {
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub parameters: Option<serde_json::Value>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/bots")
            .service(list_bots)
            .service(create_bot)
            .service(get_bot)
            .service(update_bot)
            .service(delete_bot),
    );
}

#[actix_web::get("")]
async fn list_bots() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "bots": [
            {
                "id": Uuid::new_v4(),
                "name": "Example Bot 1",
                "status": "active"
            },
            {
                "id": Uuid::new_v4(),
                "name": "Example Bot 2",
                "status": "inactive"
            }
        ]
    }))
}

#[actix_web::post("")]
async fn create_bot(bot_config: web::Json<BotConfig>) -> impl Responder {
    HttpResponse::Created().json(serde_json::json!({
        "id": Uuid::new_v4(),
        "name": bot_config.name,
        "status": "created"
    }))
}

#[actix_web::get("/{bot_id}")]
async fn get_bot(bot_id: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "id": *bot_id,
        "name": "Example Bot",
        "status": "active"
    }))
}

#[actix_web::put("/{bot_id}")]
async fn update_bot(bot_id: web::Path<Uuid>, bot_config: web::Json<BotConfig>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "id": *bot_id,
        "name": bot_config.name,
        "status": "updated"
    }))
}

#[actix_web::delete("/{bot_id}")]
async fn delete_bot(bot_id: web::Path<Uuid>) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "id": *bot_id,
        "status": "deleted"
    }))
}
