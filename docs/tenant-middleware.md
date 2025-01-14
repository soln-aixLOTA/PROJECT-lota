# Tenant Middleware Documentation

## Overview

The tenant middleware components provide essential functionality for multi-tenancy support in the LotaBots platform. These components handle tenant identification, rate limiting, usage tracking, and audit logging.

## Components

### 1. Tenant Middleware

The `TenantMiddleware` is responsible for tenant identification and validation.

#### Features
- Extracts tenant ID from various sources (headers, path, query parameters, JWT)
- Validates tenant existence and status
- Checks tenant resource limits
- Injects tenant information into request context

#### Configuration
```rust
let app = App::new()
    .wrap(TenantMiddleware::new(tenant_service.clone()))
    // ... other middleware and routes
```

#### Request Flow
1. Extracts tenant ID from request
2. Validates tenant existence
3. Checks tenant status
4. Verifies resource limits
5. Injects tenant into request context

#### Headers
- `X-Tenant-ID`: UUID of the tenant (required if not provided through other means)

### 2. Rate Limit Middleware

The `RateLimitMiddleware` enforces request rate limits based on tenant subscription tiers.

#### Features
- Per-tenant rate limiting
- Dynamic rate limits based on subscription tier
- Rate limit headers in responses
- In-memory rate limit tracking

#### Configuration
```rust
let app = App::new()
    .wrap(RateLimitMiddleware::new(tenant_service.clone()))
    // ... other middleware
```

#### Rate Limits by Tier
- Free: ~11.57 requests/second (1,000 per day)
- Professional: ~115.74 requests/second (10,000 per day)
- Enterprise: ~1,157.41 requests/second (100,000 per day)
- Custom: Configurable based on tenant settings

#### Response Headers
- `X-RateLimit-Remaining`: Number of requests remaining in the current window
- `X-RateLimit-Reset`: Unix timestamp when the rate limit resets

### 3. Metrics Middleware

The `MetricsMiddleware` collects and exposes tenant usage metrics using Prometheus.

#### Features
- Request count tracking
- Response time measurement
- GPU time tracking
- Data transfer monitoring
- Per-tenant metrics

#### Configuration
```rust
let app = App::new()
    .wrap(MetricsMiddleware::new(tenant_service.clone()))
    // ... other middleware
```

#### Metrics Collected
- `http_requests_total`: Counter of HTTP requests by tenant, method, path, and status
- `http_request_duration_seconds`: Histogram of request durations
- `gpu_time_seconds`: Histogram of GPU processing time
- `data_processed_bytes`: Counter of data transferred

#### Labels
- `tenant_id`: UUID of the tenant
- `method`: HTTP method
- `path`: Request path
- `status`: Response status code
- `direction`: Data transfer direction (in/out)
- `operation`: Type of operation (for GPU metrics)

### 4. Audit Middleware

The `AuditMiddleware` logs tenant activities for security and compliance.

#### Features
- Request/response logging
- User action tracking
- IP address logging
- User agent tracking
- Timestamp recording

#### Configuration
```rust
let app = App::new()
    .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
    // ... other middleware
```

#### Audit Log Fields
- `tenant_id`: UUID of the tenant
- `user_id`: UUID of the user (if authenticated)
- `event_type`: Type of event (request/response)
- `details`: JSON object containing:
  - `method`: HTTP method
  - `path`: Request path
  - `query`: Query parameters
  - `remote_ip`: Client IP address
  - `user_agent`: Client user agent
  - `timestamp`: Event timestamp
  - `status`: Response status (for response events)

## Middleware Chain Order

The recommended order for the middleware chain is:

```rust
let app = App::new()
    .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
    .wrap(MetricsMiddleware::new(tenant_service.clone()))
    .wrap(RateLimitMiddleware::new(tenant_service.clone()))
    .wrap(TenantMiddleware::new(tenant_service.clone()))
    // ... routes and handlers
```

This order ensures:
1. Tenant validation occurs first
2. Rate limiting is applied before processing
3. Metrics capture all middleware overhead
4. Audit logging captures complete request lifecycle

## Error Handling

The middleware components can return the following errors:

### TenantMiddleware
- 400 Bad Request: Tenant ID not found in request
- 404 Not Found: Tenant does not exist
- 403 Forbidden: Tenant is inactive or deleted
- 403 Forbidden: Resource limits exceeded

### RateLimitMiddleware
- 429 Too Many Requests: Rate limit exceeded

### MetricsMiddleware
- No errors (fails open)

### AuditMiddleware
- 500 Internal Server Error: Database error when logging

## Best Practices

1. **Tenant Identification**
   - Always include tenant ID in requests
   - Use JWT tokens for authenticated requests
   - Set custom domain for easier identification

2. **Rate Limiting**
   - Monitor rate limit headers
   - Implement client-side backoff
   - Request quota increases before limits are hit

3. **Metrics**
   - Set up alerts for abnormal patterns
   - Monitor GPU usage closely
   - Track per-tenant error rates

4. **Audit Logging**
   - Regularly review audit logs
   - Set up log retention policies
   - Monitor for suspicious patterns

## Example Usage

```rust
use actix_web::{web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize services
    let pool = setup_database().await;
    let tenant_repository = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let tenant_service = Arc::new(TenantService::new(tenant_repository));

    // Create HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(tenant_service.clone()))
            // Add middleware in recommended order
            .wrap(AuditMiddleware::new(tenant_service.clone(), pool.clone()))
            .wrap(MetricsMiddleware::new(tenant_service.clone()))
            .wrap(RateLimitMiddleware::new(tenant_service.clone()))
            .wrap(TenantMiddleware::new(tenant_service.clone()))
            // Configure routes
            .configure(tenant_handlers::configure)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Monitoring and Maintenance

1. **Prometheus Metrics**
   - Set up Prometheus to scrape metrics endpoint
   - Create Grafana dashboards for visualization
   - Configure alerts for key metrics

2. **Audit Log Analysis**
   - Use log aggregation tools
   - Set up automated reports
   - Configure security alerts

3. **Performance Monitoring**
   - Monitor middleware overhead
   - Track rate limit impact
   - Analyze tenant usage patterns

4. **Maintenance Tasks**
   - Regular audit log cleanup
   - Rate limit cache maintenance
   - Metrics aggregation and archival 