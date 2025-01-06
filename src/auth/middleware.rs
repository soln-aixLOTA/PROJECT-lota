use axum::{
<<<<<<< HEAD
    body::Body,
=======
>>>>>>> 921251a (fetch)
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashSet;

use crate::{auth::AuthUser, core::error::DocumentError};

<<<<<<< HEAD
pub async fn require_auth(
    auth: Option<AuthUser>,
    request: Request<Body>,
    next: Next,
=======
pub async fn require_auth<B>(
    auth: Option<AuthUser>,
    request: Request<B>,
    next: Next<B>,
>>>>>>> 921251a (fetch)
) -> Result<Response, DocumentError> {
    match auth {
        Some(_) => Ok(next.run(request).await),
        None => Err(DocumentError::AuthenticationError(
            "Unauthorized".to_string(),
        )),
    }
}

<<<<<<< HEAD
pub async fn require_roles(
    roles: Vec<String>,
    auth: Option<AuthUser>,
    request: Request<Body>,
    next: Next,
=======
pub async fn require_roles<B>(
    roles: Vec<String>,
    auth: Option<AuthUser>,
    request: Request<B>,
    next: Next<B>,
>>>>>>> 921251a (fetch)
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
