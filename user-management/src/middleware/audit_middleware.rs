use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use chrono::Utc;
use futures_util::future::LocalBoxFuture;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::tenant::Tenant,
    services::tenant_service::TenantService,
};

pub struct AuditMiddleware {
    tenant_service: Arc<TenantService>,
    pool: Arc<PgPool>,
}

impl AuditMiddleware {
    pub fn new(tenant_service: Arc<TenantService>, pool: PgPool) -> Self {
        Self {
            tenant_service,
            pool: Arc::new(pool),
        }
    }

    async fn log_event(
        &self,
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        event_type: &str,
        details: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (tenant_id, user_id, event_type, details)
            VALUES ($1, $2, $3, $4)
            "#,
            tenant_id,
            user_id,
            event_type,
            details
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuditMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuditMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuditMiddlewareService {
            service: Rc::new(service),
            tenant_service: self.tenant_service.clone(),
            pool: self.pool.clone(),
        }))
    }
}

pub struct AuditMiddlewareService<S> {
    service: Rc<S>,
    tenant_service: Arc<TenantService>,
    pool: Arc<PgPool>,
}

impl<S, B> Service<ServiceRequest> for AuditMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let pool = self.pool.clone();

        // Extract request information
        let method = req.method().as_str().to_string();
        let path = req.path().to_string();
        let query = req.query_string().to_string();
        let remote_ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        Box::pin(async move {
            // Get tenant and user from request extensions
            let tenant = req
                .extensions()
                .get::<Tenant>()
                .ok_or_else(|| Error::from(ServiceError::InvalidRequest(
                    "Tenant not found in request extensions".to_string(),
                )))?;

            let user_id = req.extensions().get::<Uuid>().copied();

            // Create audit event details
            let details = json!({
                "method": method,
                "path": path,
                "query": query,
                "remote_ip": remote_ip,
                "user_agent": user_agent,
                "timestamp": Utc::now().to_rfc3339(),
            });

            // Log the request
            sqlx::query!(
                r#"
                INSERT INTO audit_logs (tenant_id, user_id, event_type, details)
                VALUES ($1, $2, $3, $4)
                "#,
                tenant.id,
                user_id,
                "request",
                details
            )
            .execute(&*pool)
            .await
            .map_err(|e| Error::from(ServiceError::DatabaseError(e)))?;

            // Call the next service
            let res = service.call(req).await?;

            // Log the response
            let response_details = json!({
                "status": res.status().as_u16(),
                "timestamp": Utc::now().to_rfc3339(),
            });

            sqlx::query!(
                r#"
                INSERT INTO audit_logs (tenant_id, user_id, event_type, details)
                VALUES ($1, $2, $3, $4)
                "#,
                tenant.id,
                user_id,
                "response",
                response_details
            )
            .execute(&*pool)
            .await
            .map_err(|e| Error::from(ServiceError::DatabaseError(e)))?;

            Ok(res)
        })
    }
} 