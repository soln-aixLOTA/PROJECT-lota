use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};

use crate::{auth::validate_token, AppError};

#[allow(dead_code)]
pub struct Auth<S> {
    service: S,
}

impl<S, B> Transform<S, ServiceRequest> for Auth<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract auth header immediately (sync)
        let header_opt = req
            .headers()
            .get("Authorization")
            .and_then(|val| val.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        // If no valid Bearer token, return error right away
        let token = match header_opt {
            Some(tok) => tok,
            None => {
                let fut = async move {
                    Err(Error::from(AppError::Authentication(
                        "Missing or invalid authorization header".to_string(),
                    )))
                };
                return Box::pin(fut);
            }
        };

        // Validate token synchronously
        let claims = match validate_token(token) {
            Ok(c) => c,
            Err(e) => {
                let err_string = e.to_string();
                let fut = async move { Err(Error::from(AppError::Authentication(err_string))) };
                return Box::pin(fut);
            }
        };

        // Insert claims and call next service
        let req = req;
        req.extensions_mut().insert(claims);
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
