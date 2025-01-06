use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashSet;

use crate::{auth::AuthUser, core::error::DocumentError};

pub async fn require_auth(
    auth: Option<AuthUser>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, DocumentError> {
    match auth {
        Some(_) => Ok(next.run(request).await),
        None => Err(DocumentError::AuthenticationError(
            "Unauthorized".to_string(),
        )),
    }
}

pub async fn require_roles(
    roles: Vec<String>,
    auth: Option<AuthUser>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, DocumentError> {
    let auth =
        auth.ok_or_else(|| DocumentError::AuthenticationError("Unauthorized".to_string()))?;

    let required_roles: HashSet<_> = roles.into_iter().collect();
    let user_roles: HashSet<_> = auth.roles.into_iter().collect();

    if required_roles.is_subset(&user_roles) {
        Ok(next.run(request).await)
    } else {
        Err(DocumentError::AuthorizationError(
            "Insufficient permissions".to_string(),
        ))
    }
}
