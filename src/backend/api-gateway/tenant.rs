use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};

pub struct TenantMiddleware;

impl TenantMiddleware {
    pub fn new() -> Self {
        TenantMiddleware
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TenantMiddlewareService { service }))
    }
}

pub struct TenantMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
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
        // Extract tenant information from headers or other sources
        let tenant_id = req
            .headers()
            .get("X-Tenant-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("default")
            .to_string();

        // Store tenant information in request extensions
        req.extensions_mut().insert(tenant_id);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}
