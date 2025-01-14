use crate::auth::jwt::AuthUser;
use crate::models::document::{CreateDocumentRequest, Document, DocumentResponse};
use crate::error::AppError;
use actix_web::{get, post, web, HttpResponse, HttpRequest, FromRequest};
use sqlx::PgPool;
use validator::Validate;
use uuid::Uuid;

pub type AppResult<T> = Result<T, AppError>;

async fn authenticate(req: &HttpRequest) -> Result<AuthUser, AppError> {
    AuthUser::from_request(req, &mut actix_web::dev::Payload::None).await
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(create_document)
            .service(get_document)
            .service(list_documents),
    );
}

#[post("")]
pub async fn create_document(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    doc_req: web::Json<CreateDocumentRequest>,
) -> AppResult<HttpResponse> {
    let auth_user = authenticate(&req).await?;
    doc_req.validate()?;

    let user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|e| AppError::Authentication(format!("Invalid user ID in token: {}", e)))?;

    let document = sqlx::query_as!(
        Document,
        r#"
        INSERT INTO documents (user_id, title, content, file_path, content_type, size, metadata, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
        RETURNING id, user_id, title, content, file_path, content_type, size, metadata, status as "status: _", created_at, updated_at
        "#,
        user_id,
        doc_req.title,
        doc_req.content,
        doc_req.file_path,
        doc_req.content_type,
        doc_req.size,
        doc_req.metadata.as_ref().map(|m| serde_json::to_value(m).ok()).flatten()
    )
    .fetch_one(&**pool)
    .await?;

    Ok(HttpResponse::Created().json(DocumentResponse::from(document)))
}

#[get("/{id}")]
pub async fn get_document(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let auth_user = authenticate(&req).await?;

    // First fetch the document to check ownership
    let document = sqlx::query_as!(
        Document,
        r#"
        SELECT id, user_id, title, content, file_path, content_type, size, metadata, status as "status: _", created_at, updated_at
        FROM documents
        WHERE id = $1
        "#,
        *id
    )
    .fetch_one(&**pool)
    .await
    .map_err(|_| AppError::NotFound("Document not found".into()))?;

    // Check if user has access to this document
    if !auth_user.can_access_document(&document.user_id.to_string()) {
        return Err(AppError::Forbidden("You don't have access to this document".into()));
    }

    Ok(HttpResponse::Ok().json(DocumentResponse::from(document)))
}

#[get("")]
pub async fn list_documents(
    req: HttpRequest,
    pool: web::Data<PgPool>,
) -> AppResult<HttpResponse> {
    let auth_user = authenticate(&req).await?;
    let user_id = Uuid::parse_str(&auth_user.user_id)
        .map_err(|e| AppError::Authentication(format!("Invalid user ID in token: {}", e)))?;

    // If admin, can see all documents, otherwise only their own
    let documents = if auth_user.is_admin() {
        sqlx::query_as!(
            Document,
            r#"
            SELECT id, user_id, title, content, file_path, content_type, size, metadata, status as "status: _", created_at, updated_at
            FROM documents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&**pool)
        .await?
    } else {
        sqlx::query_as!(
            Document,
            r#"
            SELECT id, user_id, title, content, file_path, content_type, size, metadata, status as "status: _", created_at, updated_at
            FROM documents
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&**pool)
        .await?
    };

    let responses: Vec<DocumentResponse> = documents.into_iter().map(DocumentResponse::from).collect();
    Ok(HttpResponse::Ok().json(responses))
}
