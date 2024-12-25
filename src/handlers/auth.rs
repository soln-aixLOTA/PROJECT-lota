use actix_web::{web, HttpResponse, Responder, Error, HttpRequest};
use actix_web::web::{Data, Json};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::user::{User, CreateUser, DbError};

pub async fn register(
    db_pool: Data<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<impl Responder, actix_web::Error> {
    let user_data = User {
        id: Uuid::new_v4(),
        username: payload.username,
        email: payload.email,
        password_hash: payload.password,
        created_at: Some(Utc::now()),
        updated_at: Some(Utc::now()),
    };

    // ... rest of the code (insert user into the database, handle errors using DbError) ...
    match sqlx::query("INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&user_data.id)
        .bind(&user_data.username)
        .bind(&user_data.email)
        .bind(&user_data.password_hash)
        .bind(&user_data.created_at)
        .bind(&user_data.updated_at)
        .execute(db_pool.get_ref())
        .await
    {
        Ok(_) => Ok(HttpResponse::Created().json(user_data)),
        Err(e) => {
            eprintln!("Failed to create user: {:?}", e); // Log the error
            match e {
                DbError::UniqueViolation(msg) => Err(HttpResponse::Conflict().body(msg).into()),
                DbError::Other(msg) => Err(HttpResponse::InternalServerError().body(msg).into()),
                _ => Err(HttpResponse::InternalServerError().body("Failed to create user").into()),
            }
        }
    }
} 