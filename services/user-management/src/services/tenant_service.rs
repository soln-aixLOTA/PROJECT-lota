use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::ServiceError,
    models::tenant::{
        CreateTenantRequest, Tenant, TenantQuota, TenantStatus, TenantUsage, UpdateTenantRequest,
    },
    repositories::tenant_repository::TenantRepository,
};

pub struct TenantService {
    tenant_repository: Arc<dyn TenantRepository>,
}

impl TenantService {
    pub fn new(tenant_repository: Arc<dyn TenantRepository>) -> Self {
        Self { tenant_repository }
    }

    pub async fn create_tenant(
        &self,
        request: CreateTenantRequest,
    ) -> Result<Tenant, ServiceError> {
        // Validate custom domain if provided
        if let Some(domain) = &request.custom_domain {
            if let Some(_) = self.tenant_repository.get_tenant_by_domain(domain).await? {
                return Err(ServiceError::DomainAlreadyExists(domain.clone()));
            }
        }

        let tenant = self.tenant_repository.create_tenant(&request).await?;
        Ok(tenant)
    }

    pub async fn get_tenant_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>, ServiceError> {
        let tenant = self.tenant_repository.get_tenant_by_id(tenant_id).await?;
        Ok(tenant)
    }

    pub async fn get_tenant_by_domain(&self, domain: &str) -> Result<Option<Tenant>, ServiceError> {
        let tenant = self.tenant_repository.get_tenant_by_domain(domain).await?;
        Ok(tenant)
    }

    pub async fn update_tenant(
        &self,
        tenant_id: Uuid,
        request: UpdateTenantRequest,
    ) -> Result<Tenant, ServiceError> {
        // Check if tenant exists
        let existing_tenant = self
            .tenant_repository
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| ServiceError::TenantNotFound(tenant_id))?;

        // Validate custom domain if being updated
        if let Some(domain) = &request.custom_domain {
            if let Some(tenant) = self.tenant_repository.get_tenant_by_domain(domain).await? {
                if tenant.id != tenant_id {
                    return Err(ServiceError::DomainAlreadyExists(domain.clone()));
                }
            }
        }

        // Prevent updating deleted tenants
        if existing_tenant.status == TenantStatus::Deleted {
            return Err(ServiceError::TenantDeleted(tenant_id));
        }

        let tenant = self.tenant_repository.update_tenant(tenant_id, &request).await?;
        Ok(tenant)
    }

    pub async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), ServiceError> {
        // Check if tenant exists
        let existing_tenant = self
            .tenant_repository
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| ServiceError::TenantNotFound(tenant_id))?;

        // Prevent deleting already deleted tenants
        if existing_tenant.status == TenantStatus::Deleted {
            return Err(ServiceError::TenantDeleted(tenant_id));
        }

        self.tenant_repository.delete_tenant(tenant_id).await?;
        Ok(())
    }

    pub async fn list_tenants(&self) -> Result<Vec<Tenant>, ServiceError> {
        let tenants = self.tenant_repository.list_tenants().await?;
        Ok(tenants)
    }

    pub async fn get_tenant_usage(&self, tenant_id: Uuid) -> Result<Option<TenantUsage>, ServiceError> {
        // Check if tenant exists and is active
        let existing_tenant = self
            .tenant_repository
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| ServiceError::TenantNotFound(tenant_id))?;

        if existing_tenant.status != TenantStatus::Active {
            return Err(ServiceError::TenantInactive(tenant_id));
        }

        let usage = self.tenant_repository.get_tenant_usage(tenant_id).await?;
        Ok(usage)
    }

    pub async fn check_tenant_limits(&self, tenant_id: Uuid) -> Result<bool, ServiceError> {
        // Check if tenant exists and is active
        let existing_tenant = self
            .tenant_repository
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| ServiceError::TenantNotFound(tenant_id))?;

        if existing_tenant.status != TenantStatus::Active {
            return Err(ServiceError::TenantInactive(tenant_id));
        }

        let within_limits = self.tenant_repository.check_tenant_limits(tenant_id).await?;
        Ok(within_limits)
    }

    pub async fn get_tenant_quota(&self, tenant_id: Uuid) -> Result<TenantQuota, ServiceError> {
        let tenant = self
            .tenant_repository
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| ServiceError::TenantNotFound(tenant_id))?;

        Ok(tenant.get_quota())
    }
} 