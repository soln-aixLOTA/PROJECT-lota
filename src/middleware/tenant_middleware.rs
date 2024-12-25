use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use uuid::Uuid;

#[derive(Clone)]
pub struct TenantMiddleware<S> {
    pub service: S,
}

impl< S, B > Transform< S, ServiceRequest > for TenantMiddleware<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TenantMiddleware<S>;
    type InitError = ();
    type Future = Ready< Result< Self::Transform, Self::InitError > >;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok(TenantMiddleware { service })
    }
}

impl< S, B > Service<ServiceRequest> for TenantMiddleware<S>
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
        let tenant_slug = req.headers().get( "X-Tenant-Slug" );
        let pool = req.app_data::< web::Data<sqlx::PgPool> >().unwrap().clone();
        let fut = self.service.call( req );

        Box::pin( async move {
            if let Some( tenant_slug ) = tenant_slug {
                if let Ok( tenant_slug ) = tenant_slug.to_str() {
                    match sqlx::query!( "SELECT id FROM tenants WHERE slug = $1", tenant_slug )
                        .fetch_one( &*pool )
                        .await
                    {
                        Ok( row ) => {
                            let tenant_id: Uuid = row.id;
                            req.extensions_mut().insert( tenant_id );
                            return fut.await;
                        }
                        Err( e ) => {
                            log::error!( "Tenant lookup error: {}", e );
                            return Err(crate::error::CustomError::NotFound(
                                "Tenant not found".to_string()
                            ));
                        }
                    }
                }
            }
            Err(crate::error::CustomError::Unauthorized(
                "Missing or invalid tenant slug".to_string()
            ))
        } )
    }
}

impl<S> TenantMiddleware<S> {
    pub fn new(service: S) -> Self {
        TenantMiddleware { service }
    }
} 