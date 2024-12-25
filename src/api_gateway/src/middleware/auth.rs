use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    sync::Arc,
    task::{Context, Poll},
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use moka::future::Cache;
use tracing::{error, info, warn};

use crate::state::AppState;
use crate::error::ApiError;

const TOKEN_CACHE_SIZE: u64 = 10_000;
const TOKEN_CACHE_TTL: time::Duration = time::Duration::minutes(5);

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
    iat: i64,
    tenant_id: String,
    roles: Vec<String>,
}

pub struct AuthMiddleware {
    state: Arc<AppState>,
    token_cache: Cache<String, Claims>,
}

impl AuthMiddleware {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            token_cache: Cache::builder()
                .max_capacity(TOKEN_CACHE_SIZE)
                .time_to_live(TOKEN_CACHE_TTL)
                .build(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            state: self.state.clone(),
            token_cache: self.token_cache.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    state: Arc<AppState>,
    token_cache: Cache<String, Claims>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip auth for health check and metrics endpoints
        if req.path().starts_with("/health") || req.path().starts_with("/metrics") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let state = self.state.clone();
        let token_cache = self.token_cache.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            let token = match req.headers().get("Authorization") {
                Some(auth_header) => {
                    let auth_str = auth_header.to_str().map_err(|_| {
                        ApiError::Unauthorized("Invalid Authorization header".into())
                    })?;
                    
                    if !auth_str.starts_with("Bearer ") {
                        return Err(ApiError::Unauthorized("Invalid Authorization scheme".into()).into());
                    }
                    
                    auth_str[7..].to_string()
                }
                None => {
                    return Err(ApiError::Unauthorized("Missing Authorization header".into()).into());
                }
            };

            // Try to get claims from cache
            if let Some(claims) = token_cache.get(&token).await {
                // Validate expiration
                if claims.exp > OffsetDateTime::now_utc().unix_timestamp() {
                    // Add claims to request extensions
                    req.extensions_mut().insert(claims);
                    return service.call(req).await;
                }
                
                // Remove expired token from cache
                token_cache.remove(&token).await;
            }

            // Validate token
            let claims = match validate_token(&token, &state.config) {
                Ok(claims) => claims,
                Err(e) => {
                    error!("Token validation failed: {}", e);
                    return Err(ApiError::Unauthorized("Invalid token".into()).into());
                }
            };

            // Cache valid token
            token_cache.insert(token, claims.clone()).await;

            // Add claims to request extensions
            req.extensions_mut().insert(claims);

            // Call next middleware
            service.call(req).await
        })
    }
}

fn validate_token(token: &str, config: &crate::config::Config) -> Result<Claims, ApiError> {
    let key = DecodingKey::from_rsa_pem(
        config.jwt_public_key
            .as_ref()
            .ok_or_else(|| ApiError::Configuration("JWT public key not configured".into()))?
            .as_bytes(),
    ).map_err(|e| {
        error!("Failed to create decoding key: {}", e);
        ApiError::Configuration("Invalid JWT public key".into())
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    validation.required_spec_claims = vec!["exp", "iat", "sub", "tenant_id", "roles"].into_iter().collect();

    decode::<Claims>(token, &key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|e| {
            warn!("Token validation failed: {}", e);
            ApiError::Unauthorized("Invalid token".into())
        })
} 