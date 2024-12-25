use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "subscription_tier", rename_all = "snake_case")]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "tenant_status", rename_all = "snake_case")]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[sqlx(type_name = "support_level", rename_all = "snake_case")]
pub enum SupportLevel {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub subscription_tier: SubscriptionTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: TenantStatus,
    pub max_users: i32,
    pub max_bots: i32,
    pub max_requests_per_day: i32,
    pub gpu_quota_minutes: i32,
    pub custom_domain: Option<String>,
    pub support_level: SupportLevel,
    pub billing_email: String,
    pub technical_contact_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuota {
    pub max_users: i32,
    pub max_bots: i32,
    pub max_requests_per_day: i32,
    pub gpu_quota_minutes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsage {
    pub tenant_id: Uuid,
    pub current_user_count: i64,
    pub current_bot_count: i64,
    pub requests_today: i64,
    pub gpu_minutes_used: i64,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub subscription_tier: SubscriptionTier,
    pub billing_email: String,
    pub technical_contact_email: String,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub subscription_tier: Option<SubscriptionTier>,
    pub status: Option<TenantStatus>,
    pub max_users: Option<i32>,
    pub max_bots: Option<i32>,
    pub max_requests_per_day: Option<i32>,
    pub gpu_quota_minutes: Option<i32>,
    pub custom_domain: Option<String>,
    pub support_level: Option<SupportLevel>,
    pub billing_email: Option<String>,
    pub technical_contact_email: Option<String>,
}

impl Tenant {
    pub fn get_quota(&self) -> TenantQuota {
        match self.subscription_tier {
            SubscriptionTier::Free => TenantQuota {
                max_users: 5,
                max_bots: 2,
                max_requests_per_day: 1000,
                gpu_quota_minutes: 60,
            },
            SubscriptionTier::Professional => TenantQuota {
                max_users: 20,
                max_bots: 10,
                max_requests_per_day: 10000,
                gpu_quota_minutes: 300,
            },
            SubscriptionTier::Enterprise => TenantQuota {
                max_users: 100,
                max_bots: 50,
                max_requests_per_day: 100000,
                gpu_quota_minutes: 1000,
            },
            SubscriptionTier::Custom => TenantQuota {
                max_users: self.max_users,
                max_bots: self.max_bots,
                max_requests_per_day: self.max_requests_per_day,
                gpu_quota_minutes: self.gpu_quota_minutes,
            },
        }
    }
} 