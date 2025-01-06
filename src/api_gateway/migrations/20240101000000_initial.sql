<<<<<<< HEAD
version https://git-lfs.github.com/spec/v1
oid sha256:692e90e4f1283d74bcaaeee5226642579fa6e0dc9ccd9b1877bb7b4541ddc987
size 4632
=======
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
-- Create tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    api_key VARCHAR(255) UNIQUE NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    tier VARCHAR(50) NOT NULL DEFAULT 'basic',
    rate_limit INTEGER NOT NULL DEFAULT 1000,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create models table
CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, version)
);
-- Create tenant_models table for model access control
CREATE TABLE tenant_models (
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    model_id UUID REFERENCES models(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tenant_id, model_id)
);
-- Create usage_logs table
CREATE TABLE usage_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    model_id UUID REFERENCES models(id) ON DELETE CASCADE,
    request_id UUID NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    status_code INTEGER NOT NULL,
    processing_time DOUBLE PRECISION NOT NULL,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create rate_limit_logs table
CREATE TABLE rate_limit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    endpoint VARCHAR(255) NOT NULL,
    hits INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create error_logs table
CREATE TABLE error_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE
    SET NULL,
        request_id UUID,
        error_type VARCHAR(255) NOT NULL,
        error_message TEXT NOT NULL,
        stack_trace TEXT,
        created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Create indexes
CREATE INDEX idx_tenants_api_key ON tenants(api_key);
CREATE INDEX idx_models_name_version ON models(name, version);
CREATE INDEX idx_usage_logs_tenant_id ON usage_logs(tenant_id);
CREATE INDEX idx_usage_logs_model_id ON usage_logs(model_id);
CREATE INDEX idx_usage_logs_created_at ON usage_logs(created_at);
CREATE INDEX idx_rate_limit_logs_tenant_id_endpoint ON rate_limit_logs(tenant_id, endpoint);
CREATE INDEX idx_rate_limit_logs_window_start ON rate_limit_logs(window_start);
CREATE INDEX idx_error_logs_tenant_id ON error_logs(tenant_id);
CREATE INDEX idx_error_logs_created_at ON error_logs(created_at);
-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ language 'plpgsql';
-- Create triggers for updated_at
CREATE TRIGGER update_tenants_updated_at BEFORE
UPDATE ON tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_models_updated_at BEFORE
UPDATE ON models FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
-- Insert initial data
INSERT INTO tenants (name, api_key, status, tier, rate_limit)
VALUES (
        'Demo Tenant',
        'demo-key-123',
        'active',
        'basic',
        1000
    ),
    (
        'Premium Tenant',
        'premium-key-456',
        'active',
        'premium',
        5000
    );
INSERT INTO models (name, version, description, status)
VALUES (
        'gpt-3.5-turbo',
        '1.0',
        'GPT-3.5 Turbo language model',
        'active'
    ),
    ('gpt-4', '1.0', 'GPT-4 language model', 'active'),
    (
        'stable-diffusion',
        '2.1',
        'Stable Diffusion image generation model',
        'active'
    );
-- Grant model access to tenants
INSERT INTO tenant_models (tenant_id, model_id)
SELECT t.id,
    m.id
FROM tenants t
    CROSS JOIN models m
WHERE t.tier = 'premium';
INSERT INTO tenant_models (tenant_id, model_id)
SELECT t.id,
    m.id
FROM tenants t
    CROSS JOIN models m
WHERE t.tier = 'basic'
    AND m.name = 'gpt-3.5-turbo';
>>>>>>> 921251a (fetch)
