use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::document::{CreateDocumentRequest, Document, DocumentResponse, UpdateDocumentRequest};
use crate::middleware::auth::AuthMiddleware;
use actix_web::{get, post, put, web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;
use log::{debug, error};

pub type AppResult<T> = Result<T, AppError>;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/documents")
            .service(create_document)
            .service(get_document)
            .service(list_documents)
            .service(update_document)
    );
}

#[post("")]
pub async fn create_document(
    pool: web::Data<PgPool>,
    auth_user: web::ReqData<AuthUser>,
    req: web::Json<CreateDocumentRequest>,
) -> AppResult<HttpResponse> {
    req.validate()?;

    let document = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents (title, content, file_path, content_type, size, metadata, user_id, document_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, title, content, file_path, content_type, size, metadata, user_id, document_type, status, created_at, updated_at
        "#,
        req.title,
        req.content,
        req.file_path,
        req.content_type,
        req.size,
        req.metadata.as_ref().map(|m| serde_json::to_value(m).unwrap_or_default()),
        auth_user.user_id,
        req.document_type,
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(DocumentResponse::from(document)))
}

#[get("/{id}")]
pub async fn get_document(
    pool: web::Data<PgPool>,
    auth_user: web::ReqData<AuthUser>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let id = id.into_inner();

    let document = sqlx::query_as!(
        Document,
        r#"
        SELECT id, title, content, file_path, content_type, size, metadata, user_id, document_type, status, created_at, updated_at
        FROM documents
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        auth_user.user_id,
    )
    .fetch_one(&**pool)
    .await
    .map_err(|_| AppError::NotFound(format!("Document {} not found", id)))?;

    Ok(HttpResponse::Ok().json(DocumentResponse::from(document)))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListDocumentsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[get("")]
pub async fn list_documents(
    pool: web::Data<PgPool>,
    auth_user: web::ReqData<AuthUser>,
    query: web::Query<ListDocumentsQuery>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;

    let documents = sqlx::query_as!(
        Document,
        r#"
        SELECT id, title, content, file_path, content_type, size, metadata, user_id, document_type, status, created_at, updated_at
        FROM documents
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        auth_user.user_id,
        per_page,
        offset,
    )
    .fetch_all(&**pool)
    .await?;

    Ok(HttpResponse::Ok().json(documents.into_iter().map(DocumentResponse::from).collect::<Vec<_>>()))
}

#[put("/{id}")]
pub async fn update_document(
    pool: web::Data<PgPool>,
    auth_user: web::ReqData<AuthUser>,
    id: web::Path<Uuid>,
    req: web::Json<UpdateDocumentRequest>,
) -> AppResult<HttpResponse> {
    req.validate()?;
    let id = id.into_inner();

    debug!("Attempting to update document. ID: {}, User ID: {}", id, auth_user.user_id);
    debug!("Update request: {:?}", req);

    // First verify the document exists and belongs to the user
    let existing = sqlx::query_as!(
        Document,
        r#"
        SELECT id, title, content, file_path, content_type, size, metadata, user_id, document_type, status, created_at, updated_at
        FROM documents
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        auth_user.user_id,
    )
    .fetch_optional(&**pool)
    .await?;

    if let Some(existing) = existing {
        debug!("Found existing document: {:?}", existing);

        let document = sqlx::query_as!(
            Document,
            r#"
            UPDATE documents
            SET title = COALESCE($1, title),
                content = COALESCE($2, content),
                file_path = COALESCE($3, file_path),
                content_type = COALESCE($4, content_type),
                size = COALESCE($5, size),
                metadata = COALESCE($6, metadata),
                document_type = COALESCE($7, document_type),
                status = COALESCE($8, status),
                updated_at = NOW()
            WHERE id = $9 AND user_id = $10
            RETURNING id, title, content, file_path, content_type, size, metadata, user_id, document_type, status, created_at, updated_at
            "#,
            req.title,
            req.content,
            req.file_path,
            req.content_type,
            req.size,
            req.metadata.as_ref().map(|m| serde_json::to_value(m).unwrap_or_default()),
            req.document_type,
            req.status as Option<_>,
            id,
            auth_user.user_id,
        )
        .fetch_one(&**pool)
        .await
        .map_err(|e| {
            error!("Failed to update document: {:?}", e);
            AppError::NotFound(format!("Document {} not found", id))
        })?;

        debug!("Successfully updated document: {:?}", document);
        Ok(HttpResponse::Ok().json(DocumentResponse::from(document)))
    } else {
        error!("Document not found. ID: {}, User ID: {}", id, auth_user.user_id);
        Err(AppError::NotFound(format!("Document {} not found", id)))
    }
}
