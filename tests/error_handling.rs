use actix_web::{http::StatusCode, ResponseError};
use document_automation::error::AppError;

#[actix_web::test]
async fn test_error_response() {
    let error = AppError::NotFound("resource".to_string());
    let response = error.error_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = response.into_body();
    assert!(format!("{:?}", body).contains("resource"));
}
