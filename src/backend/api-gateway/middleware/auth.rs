use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, FromRequest, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub role: String,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(claims.clone())),
            None => ready(Err(ErrorUnauthorized("No valid authentication found"))),
        }
    }
}

pub struct AuthMiddleware {
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
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
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_secret: String,
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
        // Skip auth for login and register endpoints
        if req.path().ends_with("/login") || req.path().ends_with("/register") {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let auth_header = req.headers().get("Authorization");
        debug!("Auth header: {:?}", auth_header);

        let auth_str = match auth_header {
            Some(header) => match header.to_str() {
                Ok(s) => s,
                Err(_) => {
                    error!("Invalid authorization header format");
                    return Box::pin(async move {
                        Err(actix_web::error::ErrorUnauthorized(
                            "Invalid authorization header",
                        ))
                    });
                }
            },
            None => {
                error!("Missing authorization header");
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized(
                        "Missing authorization header",
                    ))
                });
            }
        };

        if !auth_str.starts_with("Bearer ") {
            error!("Invalid authorization scheme");
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized(
                    "Invalid authorization scheme",
                ))
            });
        }

        let token = &auth_str[7..];
        debug!("Token: {}", token);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Temporarily disable expiration validation

        let token_data = match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        ) {
            Ok(data) => data,
            Err(e) => {
                error!("Token validation failed: {}", e);
                return Box::pin(async move {
                    Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                });
            }
        };

        debug!("Token claims: {:?}", token_data.claims);
        req.extensions_mut().insert(token_data.claims);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}
