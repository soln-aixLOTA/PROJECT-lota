use crate::auth::validate_access_token;
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};

pub struct JwtMiddleware;

impl<S> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static + Clone,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService { service }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static + Clone,
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
        let auth_header = req.headers().get("Authorization");

        let auth_header = match auth_header {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Missing Authorization header"))
                });
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid Authorization header"))
                });
            }
        };

        let token = match auth_str.strip_prefix("Bearer ") {
            Some(token) => token.trim(),
            None => {
                return Box::pin(async move {
                    Err(ErrorUnauthorized("Invalid Authorization header format"))
                });
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