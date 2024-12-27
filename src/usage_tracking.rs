use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::task::{Context, Poll};

pub struct UsageTrackingMiddleware;

impl UsageTrackingMiddleware {
    pub fn new() -> Self {
        UsageTrackingMiddleware
    }
}

impl<S, B> Transform<S, ServiceRequest> for UsageTrackingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UsageTrackingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UsageTrackingMiddlewareService { service }))
    }
}

pub struct UsageTrackingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for UsageTrackingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            // TODO: Implement usage tracking logic here
            Ok(res)
        })
    }
}
