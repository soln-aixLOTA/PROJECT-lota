use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::utils::jwt::decode_jwt;

#[derive(Clone)]
pub struct JwtAuthMiddleware<S> {
    pub service: S,
}

impl< S, B > Transform< S, ServiceRequest > for JwtAuth
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready< Result< Self::Transform, Self::InitError > >;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok(JwtAuthMiddleware { service })
    }
}

impl< S, B > Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin< Box< dyn Future< Output = Result< Self::Response, Self::Error > > > >;

    forward_ready!(service);

    fn call( &self, mut req: ServiceRequest ) -> Self::Future {
        let auth_header_str = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_owned());

        let fut = self.service.call(req);

        Box::pin(async move {
            if let Some(auth_str) = auth_header_str {
                if auth_str.starts_with("Bearer ") {
                    let token = auth_str[7..].trim();
                    match decode_jwt(token) {
                        Ok(claims) => {
                            let user_id = claims.sub;
                            let user_uuid = Uuid::parse_str(&user_id).unwrap();
                            req.extensions_mut().insert(user_uuid);
                            return fut.await;
                        }
                        Err(e) => {
                            log::error!("JWT decode error: {}", e);
                            return Err(actix_web::error::ErrorUnauthorized(
                                "Invalid token"
                            ));
                        }
                    }
                }
            }
            Err(actix_web::error::ErrorUnauthorized(
                "Missing or invalid token"
            ))
        })
    }
}

#[derive(Clone)]
pub struct JwtAuth;

impl JwtAuth {
    pub fn new() -> Self {
        JwtAuth
    }
}
