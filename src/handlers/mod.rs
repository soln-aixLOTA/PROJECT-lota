<<<<<<< HEAD
use axum::{
    routing::{get, post},
    Router,
};

use crate::core::AppState;

pub mod documents;
pub mod workflows;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/documents", documents::routes())
        .nest("/workflows", workflows::routes())
=======
pub mod auth;
pub mod inference;
pub mod models;
pub mod users;

use actix_web::web;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(auth::routes)
            .configure(users::routes)
            .configure(models::routes)
            .configure(inference::routes),
    );
>>>>>>> 921251a (fetch)
}
