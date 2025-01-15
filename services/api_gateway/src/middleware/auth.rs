use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthMiddleware {
    jwt_secret: Rc<String>,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret: Rc::new(jwt_secret),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    jwt_secret: Rc<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let jwt_secret = self.jwt_secret.clone();
        let service = self.service.clone();

        Box::pin(async move {
            if let Some(token) = auth_header {
                let key = DecodingKey::from_secret(jwt_secret.as_bytes());
                match decode::<Claims>(&token, &key, &Validation::default()) {
                    Ok(token_data) => {
                        req.extensions_mut().insert(token_data.claims);
                        service.call(req).await
                    }
                    Err(e) => Err(ApiError::AuthenticationError(e.to_string()).into()),
                }
            } else {
                Err(ApiError::AuthenticationError("No authorization token provided".into()).into())
            }
        })
    }
}
