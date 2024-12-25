pub mod user_repository;
pub mod tenant_repository;
pub mod role_repository;
pub mod permission_repository;
pub mod audit_repository;

pub use user_repository::*;
pub use tenant_repository::*;
pub use role_repository::*;
pub use permission_repository::*;
pub use audit_repository::*;

use sqlx::PgPool;

#[derive(Clone)]
pub struct Repositories {
    pub users: Box<dyn UserRepository + Send + Sync>,
    pub tenants: Box<dyn TenantRepository + Send + Sync>,
    pub roles: Box<dyn RoleRepository + Send + Sync>,
    pub permissions: Box<dyn PermissionRepository + Send + Sync>,
    pub audit: Box<dyn AuditRepository + Send + Sync>,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            users: Box::new(PostgresUserRepository::new(pool.clone())),
            tenants: Box::new(PostgresTenantRepository::new(pool.clone())),
            roles: Box::new(PostgresRoleRepository::new(pool.clone())),
            permissions: Box::new(PostgresPermissionRepository::new(pool.clone())),
            audit: Box::new(PostgresAuditRepository::new(pool.clone())),
        }
    }
} 