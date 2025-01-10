use axum::{
    middleware::Next,
    response::Response,
    http::{Request, StatusCode},
    body::Bytes,
};
use validator::Validate;
use serde::de::DeserializeOwned;

pub async fn validate_request<T, B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, ApiError>
where
    T: DeserializeOwned + Validate,
    B: axum::body::HttpBody<Data = Bytes>,
{
    let (parts, body) = request.into_parts();
    let bytes = hyper::body::to_bytes(body).await.map_err(|_| {
        ApiError::ValidationError("Failed to read request body".to_string())
    })?;

    let value: T = serde_json::from_slice(&bytes).map_err(|e| {
        ApiError::ValidationError(format!("Invalid request format: {}", e))
    })?;

    value.validate().map_err(|e| {
        ApiError::ValidationError(e.to_string())
    })?;

    let request = Request::from_parts(parts, axum::body::Body::from(bytes));
    Ok(next.run(request).await)
} 