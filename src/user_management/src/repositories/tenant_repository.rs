use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;

use crate::models::tenant::{
    Tenant, TenantStatus, TenantUsage, CreateTenantRequest, UpdateTenantRequest,
};

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create_tenant(&self, request: &CreateTenantRequest) -> Result<Tenant, SqlxError>;
    async fn get_tenant_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>, SqlxError>;
    async fn get_tenant_by_domain(&self, domain: &str) -> Result<Option<Tenant>, SqlxError>;
    async fn update_tenant(&self, tenant_id: Uuid, request: &UpdateTenantRequest) -> Result<Tenant, SqlxError>;
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), SqlxError>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>, SqlxError>;
    async fn get_tenant_usage(&self, tenant_id: Uuid) -> Result<Option<TenantUsage>, SqlxError>;
    async fn check_tenant_limits(&self, tenant_id: Uuid) -> Result<bool, SqlxError>;
}

pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    async fn create_tenant(&self, request: &CreateTenantRequest) -> Result<Tenant, SqlxError> {
        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            INSERT INTO tenants (
                name, subscription_tier, billing_email, 
                technical_contact_email, custom_domain
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING 
                id, name, subscription_tier as "subscription_tier: _",
                created_at, updated_at, status as "status: _",
                max_users, max_bots, max_requests_per_day,
                gpu_quota_minutes, custom_domain,
                support_level as "support_level: _",
                billing_email, technical_contact_email
            "#,
            request.name,
            request.subscription_tier as _,
            request.billing_email,
            request.technical_contact_email,
            request.custom_domain,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn get_tenant_by_id(&self, tenant_id: Uuid) -> Result<Option<Tenant>, SqlxError> {
        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            SELECT 
                id, name, subscription_tier as "subscription_tier: _",
                created_at, updated_at, status as "status: _",
                max_users, max_bots, max_requests_per_day,
                gpu_quota_minutes, custom_domain,
                support_level as "support_level: _",
                billing_email, technical_contact_email
            FROM tenants
            WHERE id = $1 AND status != 'deleted'
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn get_tenant_by_domain(&self, domain: &str) -> Result<Option<Tenant>, SqlxError> {
        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            SELECT 
                id, name, subscription_tier as "subscription_tier: _",
                created_at, updated_at, status as "status: _",
                max_users, max_bots, max_requests_per_day,
                gpu_quota_minutes, custom_domain,
                support_level as "support_level: _",
                billing_email, technical_contact_email
            FROM tenants
            WHERE custom_domain = $1 AND status != 'deleted'
            "#,
            domain
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn update_tenant(
        &self,
        tenant_id: Uuid,
        request: &UpdateTenantRequest,
    ) -> Result<Tenant, SqlxError> {
        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            UPDATE tenants
            SET 
                name = COALESCE($2, name),
                subscription_tier = COALESCE($3, subscription_tier),
                status = COALESCE($4, status),
                max_users = COALESCE($5, max_users),
                max_bots = COALESCE($6, max_bots),
                max_requests_per_day = COALESCE($7, max_requests_per_day),
                gpu_quota_minutes = COALESCE($8, gpu_quota_minutes),
                custom_domain = COALESCE($9, custom_domain),
                support_level = COALESCE($10, support_level),
                billing_email = COALESCE($11, billing_email),
                technical_contact_email = COALESCE($12, technical_contact_email),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, name, subscription_tier as "subscription_tier: _",
                created_at, updated_at, status as "status: _",
                max_users, max_bots, max_requests_per_day,
                gpu_quota_minutes, custom_domain,
                support_level as "support_level: _",
                billing_email, technical_contact_email
            "#,
            tenant_id,
            request.name,
            request.subscription_tier as _,
            request.status as _,
            request.max_users,
            request.max_bots,
            request.max_requests_per_day,
            request.gpu_quota_minutes,
            request.custom_domain,
            request.support_level as _,
            request.billing_email,
            request.technical_contact_email,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(tenant)
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<(), SqlxError> {
        // Soft delete by updating status
        sqlx::query!(
            r#"
            UPDATE tenants
            SET status = 'deleted', updated_at = NOW()
            WHERE id = $1
            "#,
            tenant_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_tenants(&self) -> Result<Vec<Tenant>, SqlxError> {
        let tenants = sqlx::query_as!(
            Tenant,
            r#"
            SELECT 
                id, name, subscription_tier as "subscription_tier: _",
                created_at, updated_at, status as "status: _",
                max_users, max_bots, max_requests_per_day,
                gpu_quota_minutes, custom_domain,
                support_level as "support_level: _",
                billing_email, technical_contact_email
            FROM tenants
            WHERE status != 'deleted'
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tenants)
    }

    async fn get_tenant_usage(&self, tenant_id: Uuid) -> Result<Option<TenantUsage>, SqlxError> {
        let usage = sqlx::query_as!(
            TenantUsage,
            r#"
            WITH daily_stats AS (
                SELECT
                    COUNT(*) FILTER (WHERE event_type = 'request') as requests_today,
                    SUM(CAST(details->>'gpu_time_ms' AS BIGINT)) FILTER (WHERE event_type = 'inferencing') as gpu_minutes_used
                FROM audit_logs
                WHERE tenant_id = $1
                AND created_at >= CURRENT_DATE
            )
            SELECT
                $1 as tenant_id,
                (SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND status != 'inactive') as current_user_count,
                (SELECT COUNT(*) FROM bots WHERE tenant_id = $1 AND status = 'active') as current_bot_count,
                COALESCE(ds.requests_today, 0) as requests_today,
                COALESCE(ds.gpu_minutes_used, 0) as gpu_minutes_used,
                (SELECT MAX(created_at) FROM audit_logs WHERE tenant_id = $1) as last_activity
            FROM daily_stats ds
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(usage)
    }

    async fn check_tenant_limits(&self, tenant_id: Uuid) -> Result<bool, SqlxError> {
        let tenant = self.get_tenant_by_id(tenant_id).await?;
        
        if let Some(tenant) = tenant {
            let usage = self.get_tenant_usage(tenant_id).await?;
            
            if let Some(usage) = usage {
                let quota = tenant.get_quota();

                // Check user limit
                if usage.current_user_count >= quota.max_users as i64 {
                    return Ok(false);
                }

                // Check bot limit
                if usage.current_bot_count >= quota.max_bots as i64 {
                    return Ok(false);
                }

                // Check daily request limit
                if usage.requests_today >= quota.max_requests_per_day as i64 {
                    return Ok(false);
                }

                // Check GPU quota
                if usage.gpu_minutes_used >= quota.gpu_quota_minutes as i64 {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
} 