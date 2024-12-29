use crate::state::AppState;
use crate::{
    middleware::auth::Claims,
    models::user::{CreateUserRequest, LoginRequest},
    services::auth::AuthService,
};
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Data, Json, Path};
use actix_web::{get, post, web, HttpResponse, Result};
use serde_json::json;
use tracing::{debug, error, info};
use uuid::Uuid;

#[post("/register")]
pub async fn register(
    service: Data<AuthService>,
    request: Json<CreateUserRequest>,
) -> HttpResponse {
    match service.register(request.0).await {
        Ok(user) => {
            info!("Successfully registered new user");
            HttpResponse::Created().json(user)
        }
        Err(e) => {
            error!("Registration failed: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": e.to_string()
            }))
        }
    }
}

#[post("/login")]
pub async fn login(service: Data<AuthService>, request: Json<LoginRequest>) -> HttpResponse {
    match service.login(request.0).await {
        Ok(response) => {
            info!("Successfully logged in user");
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Login failed: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": e.to_string()
            }))
        }
    }
}

#[get("/users/me")]
pub async fn get_current_user(claims: Claims, state: Data<AppState>) -> Result<HttpResponse> {
    let user = state
        .user_repo
        .find_by_id(Uuid::parse_str(&claims.sub).map_err(|_| {
            error!("Invalid user ID format");
            ErrorInternalServerError("Invalid user ID format")
        })?)
        .await
        .map_err(|e| {
            error!("Failed to find user: {}", e);
            ErrorInternalServerError("Failed to find user")
        })?
        .ok_or_else(|| {
            error!("User not found");
            ErrorInternalServerError("User not found")
        })?;

    Ok(HttpResponse::Ok().json(user))
}
