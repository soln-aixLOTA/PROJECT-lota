use crate::auth::validate_access_token;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

#[derive(Clone)]
pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"));

        let auth_header = match auth_header {
            Ok(header) => header,
            Err(e) => return Box::pin(async move { Err(e) }),
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid Authorization header"))
                })
            }
        };

        let token = match auth_str.strip_prefix("Bearer ") {
            Some(token) => token.trim(),
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid Authorization header format"))
                })
            }
        };

        let service = self.service.clone();
        match validate_access_token(token) {
            Ok(auth_user) => {
                let mut req = req;
                req.extensions_mut().insert(auth_user);
                Box::pin(async move { service.call(req).await })
            }
            Err(_) => Box::pin(async move { Err(ErrorUnauthorized("Invalid token")) }),
        }
    }
} 