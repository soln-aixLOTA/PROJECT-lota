use super::*;
use axum::http::StatusCode;

#[tokio::test]
async fn test_api_error_response() {
    let error = ApiError::NotFound("resource".to_string());
    let response = error.into_response();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["error"]["message"], "Resource not found: resource");
    assert_eq!(json["error"]["code"], 404);
} 