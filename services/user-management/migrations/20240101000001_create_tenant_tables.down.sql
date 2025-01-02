-- Drop trigger
DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column();

-- Drop indexes
DROP INDEX IF EXISTS idx_audit_logs_created_at;
DROP INDEX IF EXISTS idx_audit_logs_event_type;
DROP INDEX IF EXISTS idx_audit_logs_user_id;
DROP INDEX IF EXISTS idx_audit_logs_tenant_id;
DROP INDEX IF EXISTS idx_tenants_custom_domain;

-- Drop tables
DROP TABLE IF EXISTS audit_logs;
DROP TABLE IF EXISTS tenants;

-- Drop enum types
DROP TYPE IF EXISTS support_level;
DROP TYPE IF EXISTS tenant_status;
DROP TYPE IF EXISTS subscription_tier; 