pub mod auth;
pub mod documents;
pub mod workflows;

use actix_web::web;
use crate::middleware::auth::Auth;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Public routes
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
    );

    // Protected routes
    cfg.service(
        web::scope("/api")
            .wrap(Auth)
            .service(
                web::scope("/documents")
                    .route("", web::post().to(documents::create_document))
                    .route("/{id}", web::get().to(documents::get_document))
            )
            .service(
                web::scope("/workflows")
                    .route("", web::post().to(workflows::create_workflow))
                    .route("/{id}", web::get().to(workflows::get_workflow))
            )
    );
}
