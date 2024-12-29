use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
use actix_web::{HttpMessage, Result};
use futures_util::future::LocalBoxFuture;
use std::future::ready;

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
                    "max-age=31536000; includeSubDomains",
                ))
                .finish();
            let (_req, _pl) = req.into_parts();
            return Ok(ServiceResponse::new(_req, response.into_body()));
        }

        service.call(req).await
    }
}
