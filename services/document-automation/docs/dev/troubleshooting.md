# Troubleshooting Guide

This guide covers common issues you might encounter when running the Document Automation Service and their solutions.

## Common Issues

### 1. Service Won't Start

#### Symptoms

- Service fails to start
- Error in logs about port binding
- Database connection errors

#### Solutions

1. **Port Already in Use**

```bash
# Check if port is in use
sudo lsof -i :8080

# Kill process using the port
sudo kill -9 <PID>
```

2. **Database Connection Issues**

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Verify connection string
psql "postgresql://user:password@localhost:5432/docautomation"

# Check database migrations
sqlx migrate info
```

3. **Permission Issues**

```bash
# Check service user permissions
sudo ls -l /opt/docautomation
sudo ls -l /opt/docautomation/config/production.toml

# Fix permissions
sudo chown -R docautomation:docautomation /opt/docautomation
sudo chmod 600 /opt/docautomation/config/production.toml
```

### 2. Document Upload Failures

#### Symptoms

- Upload requests fail with 5xx errors
- Storage backend errors in logs
- Timeout errors

#### Solutions

1. **Storage Backend Issues**

```bash
# Check S3 credentials
aws s3 ls s3://your-bucket --profile your-profile

# Verify storage configuration
cat config/production.toml | grep STORAGE

# Check storage permissions
aws iam get-user
aws iam simulate-principal-policy --policy-source-arn arn:aws:iam::ACCOUNT-ID:user/USERNAME --action-names s3:PutObject
```

2. **File Size Issues**

```nginx
# Nginx configuration for large files
client_max_body_size 100M;
```

3. **Timeout Issues**

```toml
# Adjust timeouts in config
[server]
upload_timeout = 300  # seconds

[storage]
operation_timeout = 60  # seconds
```

### 3. Authentication Problems

#### Symptoms

- JWT validation errors
- "Unauthorized" responses
- Token expiration issues

#### Solutions

1. **JWT Configuration**

```bash
# Check JWT secret is set
echo $JWT_SECRET

# Verify token expiration
grep TOKEN_EXPIRATION config/production.toml
```

2. **Token Debugging**

```bash
# Decode JWT token
echo "your.jwt.token" | jwt decode -

# Check token expiration
jwt verify "your.jwt.token" --secret "your-secret"
```

### 4. Performance Issues

#### Symptoms

- Slow response times
- High CPU/memory usage
- Database connection exhaustion

#### Solutions

1. **Resource Monitoring**

```bash
# Check system resources
top
htop
free -m

# Monitor PostgreSQL
pg_top
```

2. **Connection Pool Tuning**

```toml
[database]
max_connections = 20
min_connections = 5
max_lifetime = "30m"
idle_timeout = "10m"
```

3. **Worker Thread Optimization**

```toml
[server]
workers = 4  # Adjust based on CPU cores
```

### 5. Storage Space Issues

#### Symptoms

- Disk space warnings
- Failed document uploads
- Slow document retrieval

#### Solutions

1. **Disk Space Management**

```bash
# Check disk usage
df -h
du -sh /opt/docautomation/data/*

# Clean old logs
journalctl --vacuum-time=7d
```

2. **Database Cleanup**

```sql
-- Remove old documents
DELETE FROM documents
WHERE status = 'archived'
AND updated_at < NOW() - INTERVAL '90 days';

-- Vacuum database
VACUUM FULL documents;
```

3. **S3 Lifecycle Rules**

```bash
# Configure S3 lifecycle rules
aws s3api put-bucket-lifecycle-configuration --bucket your-bucket --lifecycle-configuration file://lifecycle.json
```

## Logging and Debugging

### 1. Enable Debug Logging

```toml
# In config/production.toml
[logging]
level = "debug"
format = "json"
```

### 2. Collect Diagnostics

```bash
# Collect service logs
journalctl -u docautomation -n 1000 > service.log

# Collect database logs
sudo tail -n 1000 /var/log/postgresql/postgresql-*.log > db.log

# Check system metrics
vmstat 1 10 > vmstat.log
iostat -x 1 10 > iostat.log
```

### 3. Profile Performance

```bash
# CPU profiling
perf record -F 99 -p $(pgrep document-automation) -g -- sleep 30
perf report

# Memory profiling
valgrind --tool=massif ./document-automation
```

## Security Issues

### 1. Audit Access Logs

```bash
# Check API access logs
grep "POST /documents" /var/log/nginx/access.log

# Review authentication attempts
grep "Authentication failed" /var/log/docautomation/service.log
```

### 2. Security Scanning

```bash
# Run security audit
cargo audit

# Scan dependencies
cargo deny check
```

### 3. TLS Certificate Issues

```bash
# Check certificate validity
openssl x509 -in /etc/ssl/certs/docautomation.crt -text -noout

# Test TLS configuration
curl -vI https://docautomation.example.com
```

## Recovery Procedures

### 1. Database Recovery

```bash
# Restore from backup
pg_restore -d docautomation backup_20240101.sql

# Check data integrity
SELECT COUNT(*) FROM documents;
SELECT COUNT(*) FROM workflows;
```

### 2. Storage Recovery

```bash
# Sync from backup bucket
aws s3 sync s3://backup-bucket/documents s3://primary-bucket/documents

# Verify file counts
aws s3 ls s3://primary-bucket/documents --recursive | wc -l
```

### 3. Service Recovery

```bash
# Restart service
sudo systemctl restart docautomation

# Check service status
sudo systemctl status docautomation

# Verify health endpoint
curl http://localhost:8080/health
```

## Support Information

### Required Information for Support Tickets

1. **System Information**

   - OS version
   - Service version
   - Configuration files
   - Recent changes

2. **Error Information**

   - Error messages
   - Stack traces
   - Relevant logs
   - Steps to reproduce

3. **Metrics**
   - Resource usage
   - Request rates
   - Error rates
   - Performance metrics
