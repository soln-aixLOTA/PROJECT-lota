-- Create enum types
CREATE TYPE subscription_tier AS ENUM ('free', 'professional', 'enterprise', 'custom');
CREATE TYPE tenant_status AS ENUM ('active', 'suspended', 'deleted');
CREATE TYPE support_level AS ENUM ('basic', 'standard', 'premium', 'enterprise');

-- Create tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    subscription_tier subscription_tier NOT NULL DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status tenant_status NOT NULL DEFAULT 'active',
    max_users INTEGER NOT NULL DEFAULT 5,
    max_bots INTEGER NOT NULL DEFAULT 2,
    max_requests_per_day INTEGER NOT NULL DEFAULT 1000,
    gpu_quota_minutes INTEGER NOT NULL DEFAULT 60,
    custom_domain VARCHAR(255) UNIQUE,
    support_level support_level NOT NULL DEFAULT 'basic',
    billing_email VARCHAR(255) NOT NULL,
    technical_contact_email VARCHAR(255) NOT NULL
);

-- Create index on custom_domain
CREATE UNIQUE INDEX idx_tenants_custom_domain ON tenants (custom_domain) WHERE custom_domain IS NOT NULL;

-- Create audit_logs table for tenant usage tracking
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID,
    event_type VARCHAR(50) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for audit_logs
CREATE INDEX idx_audit_logs_tenant_id ON audit_logs (tenant_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs (user_id);
CREATE INDEX idx_audit_logs_event_type ON audit_logs (event_type);
CREATE INDEX idx_audit_logs_created_at ON audit_logs (created_at);

-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tenants_updated_at
    BEFORE UPDATE ON tenants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 