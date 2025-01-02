# Tenant Management API Documentation

## Overview

The Tenant Management API provides endpoints for managing tenants in the LotaBots platform. It supports multi-tenancy, subscription tiers, and usage tracking.

## Base URL

```
/api/v1/tenants
```

## Authentication

All endpoints require authentication using a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

## Endpoints

### Create Tenant

Creates a new tenant in the system.

**Request**
- Method: `POST`
- Path: `/`
- Content-Type: `application/json`

```json
{
    "name": "Example Corp",
    "subscription_tier": "professional",
    "billing_email": "billing@example.com",
    "technical_contact_email": "tech@example.com",
    "custom_domain": "example.lotabots.ai"
}
```

**Response**
- Status: 201 Created
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Example Corp",
    "subscription_tier": "professional",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "status": "active",
    "max_users": 20,
    "max_bots": 10,
    "max_requests_per_day": 10000,
    "gpu_quota_minutes": 300,
    "custom_domain": "example.lotabots.ai",
    "support_level": "standard",
    "billing_email": "billing@example.com",
    "technical_contact_email": "tech@example.com"
}
```

### Get Tenant by ID

Retrieves a tenant by their UUID.

**Request**
- Method: `GET`
- Path: `/{tenant_id}`

**Response**
- Status: 200 OK
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Example Corp",
    "subscription_tier": "professional",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "status": "active",
    "max_users": 20,
    "max_bots": 10,
    "max_requests_per_day": 10000,
    "gpu_quota_minutes": 300,
    "custom_domain": "example.lotabots.ai",
    "support_level": "standard",
    "billing_email": "billing@example.com",
    "technical_contact_email": "tech@example.com"
}
```

### Get Tenant by Domain

Retrieves a tenant by their custom domain.

**Request**
- Method: `GET`
- Path: `/domain/{domain}`

**Response**
- Status: 200 OK
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Example Corp",
    "subscription_tier": "professional",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "status": "active",
    "max_users": 20,
    "max_bots": 10,
    "max_requests_per_day": 10000,
    "gpu_quota_minutes": 300,
    "custom_domain": "example.lotabots.ai",
    "support_level": "standard",
    "billing_email": "billing@example.com",
    "technical_contact_email": "tech@example.com"
}
```

### Update Tenant

Updates an existing tenant's information.

**Request**
- Method: `PUT`
- Path: `/{tenant_id}`
- Content-Type: `application/json`

```json
{
    "name": "Updated Corp",
    "subscription_tier": "enterprise",
    "status": "active",
    "max_users": 50,
    "max_bots": 20,
    "max_requests_per_day": 50000,
    "gpu_quota_minutes": 500,
    "custom_domain": "new.lotabots.ai",
    "support_level": "premium",
    "billing_email": "new.billing@example.com",
    "technical_contact_email": "new.tech@example.com"
}
```

**Response**
- Status: 200 OK
```json
{
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "name": "Updated Corp",
    "subscription_tier": "enterprise",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z",
    "status": "active",
    "max_users": 50,
    "max_bots": 20,
    "max_requests_per_day": 50000,
    "gpu_quota_minutes": 500,
    "custom_domain": "new.lotabots.ai",
    "support_level": "premium",
    "billing_email": "new.billing@example.com",
    "technical_contact_email": "new.tech@example.com"
}
```

### Delete Tenant

Soft deletes a tenant by setting their status to "deleted".

**Request**
- Method: `DELETE`
- Path: `/{tenant_id}`

**Response**
- Status: 204 No Content

### List Tenants

Retrieves a list of all active tenants.

**Request**
- Method: `GET`
- Path: `/`

**Response**
- Status: 200 OK
```json
[
    {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "name": "Example Corp",
        "subscription_tier": "professional",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "status": "active",
        "max_users": 20,
        "max_bots": 10,
        "max_requests_per_day": 10000,
        "gpu_quota_minutes": 300,
        "custom_domain": "example.lotabots.ai",
        "support_level": "standard",
        "billing_email": "billing@example.com",
        "technical_contact_email": "tech@example.com"
    }
]
```

### Get Tenant Usage

Retrieves usage statistics for a tenant.

**Request**
- Method: `GET`
- Path: `/{tenant_id}/usage`

**Response**
- Status: 200 OK
```json
{
    "tenant_id": "123e4567-e89b-12d3-a456-426614174000",
    "current_user_count": 15,
    "current_bot_count": 5,
    "requests_today": 5000,
    "gpu_minutes_used": 120,
    "last_activity": "2024-01-01T12:00:00Z"
}
```

### Get Tenant Quota

Retrieves the quota limits for a tenant based on their subscription tier.

**Request**
- Method: `GET`
- Path: `/{tenant_id}/quota`

**Response**
- Status: 200 OK
```json
{
    "max_users": 20,
    "max_bots": 10,
    "max_requests_per_day": 10000,
    "gpu_quota_minutes": 300
}
```

### Check Tenant Limits

Checks if a tenant is within their usage limits.

**Request**
- Method: `GET`
- Path: `/{tenant_id}/check-limits`

**Response**
- Status: 200 OK
```json
true
```

### User Management Endpoints

#### User Registration

**Endpoint:** `POST /api/v1/users/register`

**Request Body:**

```json
{
  "tenant_id": "uuid", // Tenant ID
  "email": "[email address removed]", // User's email address
  "password": "StrongPassword123!", // User's password
  "first_name": "John", // User's first name (optional)
  "last_name": "Doe", // User's last name (optional)
  "mfa_enabled": false // Whether to enable MFA (optional, default: false)
}
```

**Response (201 Created):**

```json
{
  "id": "uuid", // User ID
  "tenant_id": "uuid",
  "email": "[email address removed]",
  "first_name": "John",
  "last_name": "Doe",
  "created_at": "2024-01-01T10:00:00Z",
  "updated_at": "2024-01-01T10:00:00Z",
  "last_login_at": null,
  "status": "active",
  "mfa_enabled": false
}
```

**Error Responses:**

*   400 Bad Request: Invalid input data (e.g., invalid email format, password too short).
    ```json
    {
        "code": 400,
        "message": "Invalid email format"
    }
    ```
*   409 Conflict: Email already exists within the tenant.
    ```json
    {
        "code": 409,
        "message": "User already exists with email: [email address removed]"
    }
    ```
*   500 Internal Server Error: Database error or other unexpected error.

#### User Login

**Endpoint:** `POST /api/v1/auth/login`

**Request Body:**

```json
{
  "email": "[email address removed]",
  "password": "StrongPassword123!",
  "mfa_code": "123456" // Optional MFA code
}
```

**Response (200 OK):**

```json
{
  "access_token": "jwt_access_token",
  "refresh_token": "jwt_refresh_token",
  "token_type": "bearer",
  "expires_in": 3600,
  "user": {
    "id": "uuid",
    "email": "[email address removed]",
    // ... other user profile fields ...
    "roles": ["user", "some_role"],
    "permissions": ["user.read", "bot.create"]
  }
}
```

**Error Responses:**

*   401 Unauthorized: Invalid credentials or account locked.
*   401 Unauthorized: MFA required.
*   401 Unauthorized: Invalid MFA code.
*   500 Internal Server Error: Unexpected error.

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
    "code": 400,
    "message": "Invalid request: <details>"
}
```

### 401 Unauthorized
```json
{
    "code": 401,
    "message": "Invalid token"
}
```

### 403 Forbidden
```json
{
    "code": 403,
    "message": "Permission denied"
}
```

### 404 Not Found
```json
{
    "code": 404,
    "message": "Tenant not found: <tenant_id>"
}
```

### 409 Conflict
```json
{
    "code": 409,
    "message": "Domain already exists: <domain>"
}
```

### 429 Too Many Requests
```json
{
    "code": 429,
    "message": "Rate limit exceeded"
}
```

### 500 Internal Server Error
```json
{
    "code": 500,
    "message": "Internal server error"
}
```

## Subscription Tiers

The platform supports the following subscription tiers:

- **Free**
  - 5 users
  - 2 bots
  - 1,000 requests per day
  - 60 GPU minutes

- **Professional**
  - 20 users
  - 10 bots
  - 10,000 requests per day
  - 300 GPU minutes

- **Enterprise**
  - 100 users
  - 50 bots
  - 100,000 requests per day
  - 1,000 GPU minutes

- **Custom**
  - Custom limits based on agreement 

## Security Design

### Authentication

* **JWT (JSON Web Tokens):** JWTs will be used for authentication.
  * **Claims:**
    * `sub`: User ID (UUID)
    * `tenant_id`: Tenant ID (UUID)
    * `roles`: List of user's roles
    * `permissions`: List of user's permissions
    * `exp`: Expiration time (timestamp)
    * `iat`: Issued at time (timestamp)
  * **Secret Management:** The JWT secret will be stored securely in the Kubernetes Secrets as `user-management-secrets` and injected as an environment variable.
  * **Expiration:** Access tokens will have a short expiration time (e.g., 1 hour). Refresh tokens will have a longer expiration time (e.g., 7 days).
  * **Token Refresh:** The `/api/v1/auth/refresh` endpoint will be used to obtain a new access token using a valid refresh token.

### Authorization

* **Role-Based Access Control (RBAC):**
  * Users can be assigned multiple roles.
  * Roles have associated permissions.
  * Permissions define actions on specific resources (e.g., `user.create`, `bot.read`).
* **Permission Checks:**
  * The `PermissionService` will provide methods to check if a user has a specific permission or a set of permissions.
  * These checks will be performed in the service layer before executing any sensitive operations.

### Input Validation

* All user inputs will be validated using the `validator` crate to prevent common vulnerabilities:
  * Email addresses will be validated using a regular expression or a dedicated email validation library.
  * Password complexity will be enforced (minimum length, uppercase, lowercase, numbers, special characters).
  * String inputs will be validated for length and allowed characters.
  * UUIDs will be validated using the `uuid` crate.

### Rate Limiting

* The API Gateway will implement rate limiting to prevent brute-force attacks and denial-of-service attacks.
* Rate limiting will be configurable per tenant and per subscription tier.

### Audit Logging

* All security-relevant events (e.g., user registration, login, password changes, role assignments) will be logged to the `audit_logs` table.
* The audit log entries will include the timestamp, user ID (if available), tenant ID, event type, IP address, user agent, and any relevant details.