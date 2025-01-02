use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct RateLimit<S> {
    pub service: S,
}

impl< S, B > Transform< S, ServiceRequest > for RateLimit<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimit<S>;
    type InitError = ();
    type Future = Ready< Result< Self::Transform, Self::InitError > >;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok(RateLimit { service })
    }
}

impl< S, B > Service<ServiceRequest> for RateLimit<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin< Box< dyn Future< Output = Result< Self::Response, Self::Error > > > >;

    forward_ready!(service);

    fn call( &self, req: ServiceRequest ) -> Self::Future {
        let fut = self.service.call( req );

        Box::pin( async move {
            // TODO: Implement rate limiting logic here
            fut.await
        } )
    }
}

impl<S> RateLimit<S> {
    pub fn new(service: S) -> RateLimit<S> {
        RateLimit {
            service,
        }
    }
} 