use actix_web::{delete, get, post, put, web, HttpResponse};
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::tenant::{CreateTenantRequest, UpdateTenantRequest},
    services::tenant_service::TenantService,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tenants")
            .service(create_tenant)
            .service(get_tenant)
            .service(get_tenant_by_domain)
            .service(update_tenant)
            .service(delete_tenant)
            .service(list_tenants)
            .service(get_tenant_usage)
            .service(get_tenant_quota)
            .service(check_tenant_limits),
    );
}

#[post("")]
async fn create_tenant(
    tenant_service: web::Data<TenantService>,
    request: web::Json<CreateTenantRequest>,
) -> Result<HttpResponse, ServiceError> {
    let tenant = tenant_service.create_tenant(request.into_inner()).await?;
    Ok(HttpResponse::Created().json(tenant))
}

#[get("/{tenant_id}")]
async fn get_tenant(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let tenant = tenant_service
        .get_tenant_by_id(tenant_id.into_inner())
        .await?
        .ok_or_else(|| ServiceError::TenantNotFound(tenant_id.into_inner()))?;
    Ok(HttpResponse::Ok().json(tenant))
}

#[get("/domain/{domain}")]
async fn get_tenant_by_domain(
    tenant_service: web::Data<TenantService>,
    domain: web::Path<String>,
) -> Result<HttpResponse, ServiceError> {
    let tenant = tenant_service
        .get_tenant_by_domain(&domain.into_inner())
        .await?
        .ok_or_else(|| ServiceError::TenantNotFound(Uuid::nil()))?;
    Ok(HttpResponse::Ok().json(tenant))
}

#[put("/{tenant_id}")]
async fn update_tenant(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
    request: web::Json<UpdateTenantRequest>,
) -> Result<HttpResponse, ServiceError> {
    let tenant = tenant_service
        .update_tenant(tenant_id.into_inner(), request.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(tenant))
}

#[delete("/{tenant_id}")]
async fn delete_tenant(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    tenant_service.delete_tenant(tenant_id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("")]
async fn list_tenants(
    tenant_service: web::Data<TenantService>,
) -> Result<HttpResponse, ServiceError> {
    let tenants = tenant_service.list_tenants().await?;
    Ok(HttpResponse::Ok().json(tenants))
}

#[get("/{tenant_id}/usage")]
async fn get_tenant_usage(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let usage = tenant_service
        .get_tenant_usage(tenant_id.into_inner())
        .await?
        .ok_or_else(|| ServiceError::TenantNotFound(tenant_id.into_inner()))?;
    Ok(HttpResponse::Ok().json(usage))
}

#[get("/{tenant_id}/quota")]
async fn get_tenant_quota(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let quota = tenant_service.get_tenant_quota(tenant_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(quota))
}

#[get("/{tenant_id}/check-limits")]
async fn check_tenant_limits(
    tenant_service: web::Data<TenantService>,
    tenant_id: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let within_limits = tenant_service.check_tenant_limits(tenant_id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(within_limits))
} 