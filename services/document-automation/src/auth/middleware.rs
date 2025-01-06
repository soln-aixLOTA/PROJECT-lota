use std::{future::Future, pin::Pin};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{auth::AuthUser, core::AppState};

pub async fn require_auth<B>(
    auth: Option<AuthUser>,
    request: Request<B>,
    next: Next,
) -> Result<Response, Response> {
    match auth {
        Some(_) => Ok(next.run(request).await),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Authentication required"})),
        )
            .into_response()),
    }
}

pub fn require_roles(
    allowed_roles: &'static [&'static str],
) -> impl Fn(
    Option<AuthUser>,
    Request<Body>,
    Next,
) -> Pin<Box<dyn Future<Output = Result<Response, Response>> + Send>>
       + Clone {
    let allowed_roles = allowed_roles.to_vec();
    move |auth: Option<AuthUser>, request: Request<Body>, next: Next| {
        let allowed_roles = allowed_roles.clone();
        Box::pin(async move {
            let auth = match auth {
                Some(auth) => auth,
                None => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Authentication required"})),
                    )
                        .into_response())
                }
            };

            if !auth
                .roles
                .iter()
                .any(|role| allowed_roles.contains(&role.as_str()))
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({"error": "Insufficient permissions"})),
                )
                    .into_response());
            }

            Ok(next.run(request).await)
        })
    }
}
