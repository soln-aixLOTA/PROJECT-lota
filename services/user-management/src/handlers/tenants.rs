use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::Database,
    error::{Result, UserError},
    models::{CreateTenantRequest, Tenant, UpdateTenantRequest},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tenants")
            .route("", web::post().to(create_tenant))
            .route("", web::get().to(list_tenants))
            .route("/{id}", web::get().to(get_tenant))
            .route("/{id}", web::put().to(update_tenant))
            .route("/{id}", web::delete().to(delete_tenant)),
    );
}

async fn create_tenant(
    db: web::Data<Database>,
    request: web::Json<CreateTenantRequest>,
) -> Result<HttpResponse> {
    request.validate().map_err(|e| UserError::Validation(e.to_string()))?;

    // Check if domain is unique if provided
    if let Some(domain) = &request.domain {
        let exists = sqlx::query!(
            r#"
            SELECT EXISTS(SELECT 1 FROM tenants WHERE domain = $1) as exists
            "#,
            domain
        )
        .fetch_one(db.get_pool())
        .await
        .map_err(UserError::Database)?
        .exists
        .unwrap_or(false);

        if exists {
            return Err(UserError::Conflict(format!(
                "Tenant with domain {} already exists",
                domain
            )));
        }
    }

    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        INSERT INTO tenants (name, domain)
        VALUES ($1, $2)
        RETURNING id, name, domain, is_active, created_at, updated_at
        "#,
        request.name,
        request.domain,
    )
    .fetch_one(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Created().json(tenant))
}

async fn list_tenants(db: web::Data<Database>) -> Result<HttpResponse> {
    let tenants = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, name, domain, is_active, created_at, updated_at
        FROM tenants
        WHERE is_active = true
        "#,
    )
    .fetch_all(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Ok().json(tenants))
}

async fn get_tenant(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, name, domain, is_active, created_at, updated_at
        FROM tenants
        WHERE id = $1 AND is_active = true
        "#,
        id.into_inner(),
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::NotFound("Tenant not found".to_string()))?;

    Ok(HttpResponse::Ok().json(tenant))
}

async fn update_tenant(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
    request: web::Json<UpdateTenantRequest>,
) -> Result<HttpResponse> {
    request.validate().map_err(|e| UserError::Validation(e.to_string()))?;

    let mut tenant = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, name, domain, is_active, created_at, updated_at
        FROM tenants
        WHERE id = $1 AND is_active = true
        "#,
        id.into_inner(),
    )
    .fetch_optional(db.get_pool())
    .await
    .map_err(UserError::Database)?
    .ok_or_else(|| UserError::NotFound("Tenant not found".to_string()))?;

    if let Some(name) = &request.name {
        tenant.name = name.clone();
    }
    if let Some(domain) = &request.domain {
        tenant.domain = Some(domain.clone());
    }
    if let Some(is_active) = request.is_active {
        tenant.is_active = is_active;
    }

    let updated_tenant = sqlx::query_as!(
        Tenant,
        r#"
        UPDATE tenants
        SET name = $1, domain = $2, is_active = $3, updated_at = NOW()
        WHERE id = $4
        RETURNING id, name, domain, is_active, created_at, updated_at
        "#,
        tenant.name,
        tenant.domain,
        tenant.is_active,
        tenant.id,
    )
    .fetch_one(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::Ok().json(updated_tenant))
}

async fn delete_tenant(
    db: web::Data<Database>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse> {
    sqlx::query!(
        r#"
        UPDATE tenants
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        "#,
        id.into_inner(),
    )
    .execute(db.get_pool())
    .await
    .map_err(UserError::Database)?;

    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_rt::test]
    async fn test_create_tenant() {
        // Add test implementation
    }

    #[actix_rt::test]
    async fn test_get_tenant() {
        // Add test implementation
    }
} 