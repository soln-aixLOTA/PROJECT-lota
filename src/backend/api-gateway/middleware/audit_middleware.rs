use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use uuid::Uuid;

use crate::error::CustomError;

#[derive(Clone)]
pub struct AuditMiddleware<S> {
    pub service: S,
    pub pool: Arc<sqlx::PgPool>,
}

impl< S, B > Transform< S, ServiceRequest > for AuditMiddleware<S>
where
    S: Service< ServiceRequest, Response = ServiceResponse<B>, Error = Error >,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuditMiddleware<S>;
    type InitError = ();
    type Future = Ready< Result< Self::Transform, Self::InitError > >;

    fn new_transform( &self, service: S ) -> Self::Future {
        ok(AuditMiddleware {
            service,
            pool: self.pool.clone(),
        })
    }
}

impl< S, B > Service<ServiceRequest> for AuditMiddleware<S>
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
        let pool = self.pool.clone();
        let extensions = req.extensions();
        let fut = self.service.call( req );

        Box::pin( async move {
            let res = fut.await?;
            let request_path = req.uri().path().to_string();
            let method = req.method().as_str().to_string();
            let status_code = res.status().as_u16() as i32;
            let tenant_id = extensions.get::<Uuid>().cloned();
            let user_id = extensions.get::<Uuid>().cloned();

            if let Some(tenant) = tenant_id {
                if let Some(user) = user_id {
                    if let Err(e) = sqlx::query!(
                        "INSERT INTO audit_logs (tenant_id, user_id, path, method, status_code) VALUES ($1, $2, $3, $4, $5)",
                        tenant,
                        user,
                        request_path,
                        method,
                        status_code
                    )
                    .execute(&*pool)
                    .await
                    {
                        // Convert sqlx::Error to CustomError
                        log::error!("Database error while logging audit: {}", e);
                        return Err(actix_web::Error::from(CustomError::InternalServerError(e.to_string())));
                    }
                }
            }
            Ok(res)
        } )
    }
}

impl<S> AuditMiddleware<S> {
    pub fn new(pool: Arc<sqlx::PgPool>) -> Self {
        AuditMiddleware {
            service: S::default(),
            pool,
        }
    }
} 