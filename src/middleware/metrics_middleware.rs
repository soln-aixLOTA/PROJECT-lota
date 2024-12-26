use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct Metrics<S> {
    pub service: S,
}

impl< S, B > Transform< S, ServiceRequest > for Metrics<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = Metrics<S>;
    type InitError = ();
    type Future = Ready< Result< Self::Transform, Self::InitError > >;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok(Metrics { service })
    }
}

impl< S, B > Service<ServiceRequest> for Metrics<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin< Box< dyn Future< Output = Result< Self::Response, Self::Error > > > > >;

    forward_ready!(service);

    fn call( &self, req: ServiceRequest ) -> Self::Future {
        let fut = self.service.call( req );

        Box::pin( async move {
            // TODO: Implement metrics collection here
            fut.await
        } )
    }
}

impl<S> Metrics<S> {
    pub fn new() -> Metrics<S> {
        Metrics::<S> { service: PhantomData }
    }
} 