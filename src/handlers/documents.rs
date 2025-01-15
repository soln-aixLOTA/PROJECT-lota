use crate::auth::Claims;
use crate::error::AppError;
use crate::models::document::{CreateDocumentRequest, Document, UpdateDocumentRequest};
use actix_web::{get, post, put, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[post("")]
pub async fn create_document(
    db: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    req: web::Json<CreateDocumentRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let document = Document::create(db.get_ref(), user_id, req.into_inner()).await?;
    Ok(HttpResponse::Created().json(document))
}

#[get("/{id}")]
pub async fn get_document(
    db: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let document = Document::get(db.get_ref(), user_id, *id).await?;
    Ok(HttpResponse::Ok().json(document))
}

#[get("")]
pub async fn list_documents(
    db: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    query: web::Query<ListDocumentsQuery>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let documents = Document::list(db.get_ref(), user_id, page, per_page).await?;
    Ok(HttpResponse::Ok().json(documents))
}

#[put("/{id}")]
pub async fn update_document(
    db: web::Data<PgPool>,
    claims: web::ReqData<Claims>,
    id: web::Path<Uuid>,
    req: web::Json<UpdateDocumentRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let document = Document::update(db.get_ref(), user_id, *id, req.into_inner()).await?;
    Ok(HttpResponse::Ok().json(document))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/documents")
            .service(create_document)
            .service(get_document)
            .service(list_documents)
            .service(update_document),
    );
}
