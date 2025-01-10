# LotaBots Database Schema - User Management Service

## Overview
This document describes the database schema for the User Management Service, designed to support multi-tenancy, role-based access control, and subscription-based features of the LotaBots platform.

## Tables

### tenants
```sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    subscription_tier VARCHAR(50) NOT NULL, -- 'free', 'pro', 'enterprise'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    max_users INTEGER,
    max_bots INTEGER,
    max_requests_per_day INTEGER,
    gpu_quota_minutes INTEGER,
    custom_domain VARCHAR(255),
    support_level VARCHAR(50) NOT NULL DEFAULT 'community',
    billing_email VARCHAR(255) NOT NULL,
    technical_contact_email VARCHAR(255),
    CONSTRAINT valid_subscription_tier CHECK (subscription_tier IN ('free', 'pro', 'enterprise')),
    CONSTRAINT valid_status CHECK (status IN ('active', 'suspended', 'deleted')),
    CONSTRAINT valid_support_level CHECK (support_level IN ('community', 'email', 'priority'))
);

CREATE INDEX idx_tenants_subscription_tier ON tenants(subscription_tier);
CREATE INDEX idx_tenants_status ON tenants(status);
```

### users
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    mfa_secret VARCHAR(255),
    password_reset_token UUID,
    password_reset_expires_at TIMESTAMPTZ,
    login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    CONSTRAINT valid_user_status CHECK (status IN ('active', 'inactive', 'suspended')),
    CONSTRAINT unique_email_per_tenant UNIQUE (tenant_id, email)
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
```

### roles
```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_system_role BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT unique_role_name_per_tenant UNIQUE (tenant_id, name)
);

CREATE INDEX idx_roles_tenant_id ON roles(tenant_id);
```

### permissions
```sql
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    resource_type VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_permission_name UNIQUE (name)
);
```

### role_permissions
```sql
CREATE TABLE role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id),
    permission_id UUID NOT NULL REFERENCES permissions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, permission_id)
);

CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);
```

### user_roles
```sql
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
```

### tenant_settings
```sql
CREATE TABLE tenant_settings (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    sso_enabled BOOLEAN NOT NULL DEFAULT false,
    sso_provider VARCHAR(50),
    sso_config JSONB,
    password_policy JSONB NOT NULL DEFAULT '{"min_length": 8, "require_uppercase": true, "require_numbers": true, "require_special": true}',
    mfa_required BOOLEAN NOT NULL DEFAULT false,
    session_timeout_minutes INTEGER NOT NULL DEFAULT 60,
    max_login_attempts INTEGER NOT NULL DEFAULT 5,
    lockout_duration_minutes INTEGER NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### audit_logs
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID REFERENCES users(id),
    event_type VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_action CHECK (action IN ('create', 'read', 'update', 'delete', 'login', 'logout', 'failed_login'))
);

CREATE INDEX idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
```

## Default Data

### Default Permissions
```sql
INSERT INTO permissions (name, description, resource_type, action) VALUES
('user.create', 'Create new users', 'user', 'create'),
('user.read', 'View user details', 'user', 'read'),
('user.update', 'Update user details', 'user', 'update'),
('user.delete', 'Delete users', 'user', 'delete'),
('role.create', 'Create new roles', 'role', 'create'),
('role.read', 'View role details', 'role', 'read'),
('role.update', 'Update role details', 'role', 'update'),
('role.delete', 'Delete roles', 'role', 'delete'),
('bot.create', 'Create new bots', 'bot', 'create'),
('bot.read', 'View bot details', 'bot', 'read'),
('bot.update', 'Update bot details', 'bot', 'update'),
('bot.delete', 'Delete bots', 'bot', 'delete');
```

### Default Roles
```sql
-- System roles will be created for each tenant upon tenant creation
INSERT INTO roles (name, description, is_system_role) VALUES
('tenant_admin', 'Full access to all tenant resources', true),
('user_manager', 'Can manage users and roles', true),
('bot_manager', 'Can manage bots', true),
('bot_user', 'Can use bots', true);
```

## Schema Design Considerations

### Multi-Tenancy
- All user-related data is associated with a tenant_id
- Unique constraints ensure data isolation between tenants
- Indexes on tenant_id for efficient querying

### Security
- Passwords stored as secure hashes
- Support for MFA
- Audit logging for all important actions
- Password reset functionality
- Account lockout protection

### Scalability
- UUID primary keys for distributed systems
- Appropriate indexing for common queries
- Partitioning-friendly design (tenant_id based)

### Business Requirements
- Support for different subscription tiers
- Flexible role-based access control
- Comprehensive audit logging
- Configurable tenant settings
- Support for usage tracking and quotas 