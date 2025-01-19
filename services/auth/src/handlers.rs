use actix_web::{web, HttpResponse, Responder};
use crate::{AuthResponse, LoginRequest, RegisterRequest, service::AuthService};

pub async fn register(
    req: web::Json<RegisterRequest>,
    service: web::Data<AuthService>,
) -> impl Responder {
    match service.register(req.into_inner()).await {
        Ok(token) => HttpResponse::Created().json(AuthResponse { token }),
        Err(e) => HttpResponse::BadRequest().json(e.to_string()),
    }
}

pub async fn login(
    req: web::Json<LoginRequest>,
    service: web::Data<AuthService>,
) -> impl Responder {
    match service.login(req.into_inner()).await {
        Ok(token) => HttpResponse::Ok().json(AuthResponse { token }),
        Err(e) => HttpResponse::Unauthorized().json(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/auth")
                .route("/register", web::post().to(register))
                .route("/login", web::post().to(login))
        );
}
