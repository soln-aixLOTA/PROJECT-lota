use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;

use crate::errors::Error;
use crate::models::{CreateUser, UpdateUser, UserResponse};
use crate::repositories::UserRepository;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(create_user))
            .route("/{id}", web::get().to(get_user))
            .route("/{id}", web::put().to(update_user))
            .route("/{id}", web::delete().to(delete_user)),
    );
}

async fn create_user(
    repo: web::Data<UserRepository>,
    user: web::Json<CreateUser>,
) -> Result<impl Responder, Error> {
    let user = repo.create(user.into_inner()).await?;
    Ok(HttpResponse::Created().json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

async fn get_user(
    repo: web::Data<UserRepository>,
    id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let user = repo
        .find_by_id(id.into_inner())
        .await?
        .ok_or_else(|| Error::NotFound("User not found".into()))?;

    Ok(HttpResponse::Ok().json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

async fn update_user(
    repo: web::Data<UserRepository>,
    id: web::Path<Uuid>,
    user: web::Json<UpdateUser>,
) -> Result<impl Responder, Error> {
    let user = repo.update(id.into_inner(), user.into_inner()).await?;
    Ok(HttpResponse::Ok().json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }))
}

async fn delete_user(
    repo: web::Data<UserRepository>,
    id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    repo.delete(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
