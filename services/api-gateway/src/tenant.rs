use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Future, Ready};
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tracing::{error, info, warn};
use uuid::Uuid;

lazy_static! {
    static ref TENANT_METRICS: CounterVec = register_counter_vec!(
        "api_tenant_requests",
        "API requests by tenant",
        &["tenant_id", "status"]
    )
    .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
    pub status: TenantStatus,
    pub tier: String,
    pub rate_limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Inactive,
}

impl std::fmt::Display for TenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantStatus::Active => write!(f, "active"),
            TenantStatus::Suspended => write!(f, "suspended"),
            TenantStatus::Inactive => write!(f, "inactive"),
        }
    }
}

#[derive(Clone)]
pub struct TenantMiddleware {
    db_pool: Arc<PgPool>,
}

impl TenantMiddleware {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
        }
    }

    async fn get_tenant_by_api_key(&self, api_key: &str) -> Result<Option<Tenant>, sqlx::Error> {
        sqlx::query_as!(
            Tenant,
            r#"
            SELECT 
                id,
                name,
                api_key,
                status as "status: TenantStatus",
                tier,
                rate_limit
            FROM tenants 
            WHERE api_key = $1
            "#,
            api_key
        )
        .fetch_optional(&*self.db_pool)
        .await
    }

    async fn validate_tenant(&self, api_key: &str) -> Result<Tenant, actix_web::Error> {
        match self.get_tenant_by_api_key(api_key).await {
            Ok(Some(tenant)) => {
                if tenant.status != TenantStatus::Active {
                    TENANT_METRICS
                        .with_label_values(&[&tenant.id.to_string(), "inactive"])
                        .inc();

                    warn!(
                        tenant_id = %tenant.id,
                        status = %tenant.status,
                        "Tenant is not active"
                    );

                    return Err(actix_web::error::ErrorForbidden(format!(
                        "Tenant {} is {}",
                        tenant.id, tenant.status
                    )));
                }

                TENANT_METRICS
                    .with_label_values(&[&tenant.id.to_string(), "active"])
                    .inc();

                info!(
                    tenant_id = %tenant.id,
                    tier = %tenant.tier,
                    "Tenant request validated"
                );

                Ok(tenant)
            }
            Ok(None) => {
                TENANT_METRICS
                    .with_label_values(&["unknown", "invalid"])
                    .inc();

                warn!(api_key = %api_key, "Invalid API key");
                Err(actix_web::error::ErrorUnauthorized("Invalid API key"))
            }
            Err(e) => {
                error!(error = ?e, "Database error while validating tenant");
                Err(actix_web::error::ErrorInternalServerError(
                    "Error validating tenant",
                ))
            }
        }
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
        ok(TenantMiddlewareService {
            service,
            db_pool: self.db_pool.clone(),
        })
    }
}

pub struct TenantMiddlewareService<S> {
    service: S,
    db_pool: Arc<PgPool>,
}

impl<S, B> Service<ServiceRequest> for TenantMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key = req
            .headers()
            .get("X-API-Key")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        let middleware = TenantMiddleware::new((*self.db_pool).clone());
        let fut = self.service.call(req);

        Box::pin(async move {
            let tenant = middleware.validate_tenant(api_key).await?;
            let mut response = fut.await?;

            // Add tenant information to response headers
            response
                .headers_mut()
                .insert("X-Tenant-ID", tenant.id.to_string().parse().unwrap());
            response
                .headers_mut()
                .insert("X-Tenant-Tier", tenant.tier.parse().unwrap());

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::{web, App, HttpResponse};
    use sqlx::postgres::PgPoolOptions;

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[actix_web::test]
    async fn test_tenant_middleware() {
        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://localhost/test_db")
            .await
            .expect("Failed to connect to database");

        let app = test::init_service(
            App::new()
                .wrap(TenantMiddleware::new(db_pool.clone()))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // Test with valid API key
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-API-Key", "valid_key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test with invalid API key
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-API-Key", "invalid_key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);

        // Test without API key
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
    }
}
