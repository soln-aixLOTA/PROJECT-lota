use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    services::tenant_service::TenantService,
};

pub struct TenantMiddleware {
    tenant_service: Arc<TenantService>,
}

impl TenantMiddleware {
    pub fn new(tenant_service: Arc<TenantService>) -> Self {
        Self { tenant_service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TenantMiddlewareService {
            service: Rc::new(service),
            tenant_service: self.tenant_service.clone(),
        }))
    }
}

pub struct TenantMiddlewareService<S> {
    service: Rc<S>,
    tenant_service: Arc<TenantService>,
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let tenant_service = self.tenant_service.clone();

        Box::pin(async move {
            // Extract tenant ID from request (e.g., from header, path, or token)
            let tenant_id = extract_tenant_id(&req)?;

            // Check if tenant exists and is active
            let tenant = tenant_service
                .get_tenant_by_id(tenant_id)
                .await
                .map_err(|e| Error::from(e))?
                .ok_or_else(|| Error::from(ServiceError::TenantNotFound(tenant_id)))?;

            // Check tenant limits
            let within_limits = tenant_service
                .check_tenant_limits(tenant_id)
                .await
                .map_err(|e| Error::from(e))?;

            if !within_limits {
                return Err(Error::from(ServiceError::ResourceLimitExceeded(
                    "Tenant has exceeded their resource limits".to_string(),
                )));
            }

            // Store tenant in request extensions for later use
            req.extensions_mut().insert(tenant);

            // Forward the request to the next middleware or handler
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

fn extract_tenant_id(req: &ServiceRequest) -> Result<Uuid, Error> {
    // Try to extract tenant ID from various sources
    
    // 1. Try from path parameters
    if let Some(tenant_id) = req
        .match_info()
        .get("tenant_id")
        .and_then(|id| Uuid::parse_str(id).ok())
    {
        return Ok(tenant_id);
    }

    // 2. Try from query parameters
    if let Some(tenant_id) = req
        .query_string()
        .split('&')
        .find(|param| param.starts_with("tenant_id="))
        .and_then(|param| param.split('=').nth(1))
        .and_then(|id| Uuid::parse_str(id).ok())
    {
        return Ok(tenant_id);
    }

    // 3. Try from headers
    if let Some(tenant_id) = req
        .headers()
        .get("X-Tenant-ID")
        .and_then(|id| id.to_str().ok())
        .and_then(|id| Uuid::parse_str(id).ok())
    {
        return Ok(tenant_id);
    }

    // 4. Try from JWT token claims (assuming token is already validated)
    if let Some(claims) = req.extensions().get::<JwtClaims>() {
        return Ok(claims.tenant_id);
    }

    Err(Error::from(ServiceError::InvalidRequest(
        "Tenant ID not found in request".to_string(),
    )))
}

// JWT claims structure (to be moved to a separate auth module)
#[derive(Debug)]
struct JwtClaims {
    tenant_id: Uuid,
    // Add other claims as needed
} 