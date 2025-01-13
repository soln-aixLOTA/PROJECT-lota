use actix_web::web;
use crate::middleware::auth::AuthMiddleware;

pub mod auth;
pub mod documents;
pub mod health;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(web::scope("/health").service(health::health_check))
            .service(
                web::scope("/auth")
                    .service(auth::register)
                    .service(auth::login)
                    .service(auth::refresh_token)
            )
            .service(
                web::scope("/documents")
                    .wrap(AuthMiddleware)
                    .service(documents::create_document)
                    .service(documents::get_document)
                    .service(documents::list_documents)
                    .service(documents::update_document)
            ),
    );
}
