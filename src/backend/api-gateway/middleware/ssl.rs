use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
use actix_web::{HttpMessage, Result};
use futures_util::future::LocalBoxFuture;
use std::future::ready;

pub struct SecurityHeaders;

impl<S, B> Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>
    for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    forward_ready!(service);

    async fn call(&self, req: ServiceRequest) -> Result<ServiceResponse<B>> {
        let mut res = service.call(req).await?;

        // Add security headers
        res.headers_mut().insert(
            "Strict-Transport-Security",
            "max-age=31536000; includeSubDomains; preload"
                .parse()
                .unwrap(),
        );
        res.headers_mut()
            .insert("X-Frame-Options", "SAMEORIGIN".parse().unwrap());
        res.headers_mut()
            .insert("X-Content-Type-Options", "nosniff".parse().unwrap());
        res.headers_mut()
            .insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
        res.headers_mut().insert(
            "Referrer-Policy",
            "strict-origin-when-cross-origin".parse().unwrap(),
        );
        res.headers_mut()
            .insert(
                "Content-Security-Policy",
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline';"
                    .parse()
                    .unwrap(),
            );
        res.headers_mut().insert(
            "Permissions-Policy",
            "geolocation=(), microphone=(), camera=()".parse().unwrap(),
        );

        Ok(res)
    }
}

pub struct EnforceHttps;

impl<S, B> Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>
    for EnforceHttps
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    forward_ready!(service);

    async fn call(&self, req: ServiceRequest) -> Result<ServiceResponse<B>> {
        if req.connection_info().scheme() != "https" {
            let url = req.url();
            let https_url = format!("https://{}{}", req.connection_info().host(), url.path());
            let response = actix_web::HttpResponse::Found()
                .append_header(("Location", https_url))
                .append_header((
                    "Strict-Transport-Security",
                    "max-age=31536000; includeSubDomains; preload",
                ))
                .finish();
            let (_req, _pl) = req.into_parts();
            return Ok(ServiceResponse::new(_req, response.into_body()));
        }

        service.call(req).await
    }
}
