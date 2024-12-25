use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use awc::Client;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

#[derive(Debug, Serialize, Deserialize)]
struct ContentModerationRequest {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentModerationResponse {
    is_safe: bool,
    reason: Option<String>,
}

pub struct ContentModeratorMiddleware;

impl ContentModeratorMiddleware {
    pub fn new() -> Self {
        ContentModeratorMiddleware
    }
}

impl<S, B: 'static> Transform<S, ServiceRequest> for ContentModeratorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ContentModeratorMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContentModeratorMiddlewareInner { service }))
    }
}

pub struct ContentModeratorMiddlewareInner<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContentModeratorMiddlewareInner<S>
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
        let future = self.service.call(req);
        Box::pin(async move {
            // For now, just pass through all requests
            // TODO: Implement content moderation logic
            future.await
        })
    }
}
