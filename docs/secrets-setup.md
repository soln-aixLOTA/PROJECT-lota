# Secrets Management Guide

## GitHub Secrets Setup

1. Navigate to your GitHub repository's settings
2. Go to "Secrets and variables" → "Actions"
3. Add the following repository secrets:

```bash
# Database Configuration
POSTGRES_USER=your-db-user
POSTGRES_PASSWORD=your-db-password
POSTGRES_DB=your-db-name
DATABASE_URL=postgresql://user:pass@host:5432/dbname

# Authentication
JWT_SECRET=your-secure-jwt-secret

# API Keys
HUGGING_FACE_HUB_TOKEN=your-huggingface-token
OPENAI_API_KEY=your-openai-key

# Google Cloud Configuration
GCP_PROJECT=your-gcp-project-id
GCP_SA_KEY=your-service-account-key-json
GKE_CLUSTER=your-gke-cluster-name
GKE_ZONE=your-gke-zone
```

## Google Cloud Secret Manager Setup

1. Enable the Secret Manager API:
```bash
gcloud services enable secretmanager.googleapis.com
```

2. Create and configure secrets:
```bash
# Create secrets
for SECRET in DATABASE_URL JWT_SECRET HUGGING_FACE_HUB_TOKEN OPENAI_API_KEY; do
  gcloud secrets create $SECRET --replication-policy="automatic"
done

# Add secret values (replace with your actual values)
echo -n "postgresql://user:pass@host:5432/dbname" | \
  gcloud secrets versions add DATABASE_URL --data-file=-
echo -n "your-secure-jwt-secret" | \
  gcloud secrets versions add JWT_SECRET --data-file=-
echo -n "your-huggingface-token" | \
  gcloud secrets versions add HUGGING_FACE_HUB_TOKEN --data-file=-
echo -n "your-openai-key" | \
  gcloud secrets versions add OPENAI_API_KEY --data-file=-
```

3. Grant access to service accounts:
```bash
# Get your service account email
export SA_EMAIL=$(gcloud iam service-accounts list \
  --filter="name:lotaai" \
  --format="value(email)")

# Grant Secret Manager access
for SECRET in DATABASE_URL JWT_SECRET HUGGING_FACE_HUB_TOKEN OPENAI_API_KEY; do
  gcloud secrets add-iam-policy-binding $SECRET \
    --member="serviceAccount:$SA_EMAIL" \
    --role="roles/secretmanager.secretAccessor"
done
```

## Environment-Specific Secrets

### Development
- Use `.env.local` for local development (not committed to git)
- Use GitHub repository secrets for CI/CD

### Staging
- Use Google Cloud Secret Manager
- Prefix secrets with `staging-`
- Configure separate service account

### Production
- Use Google Cloud Secret Manager
- Prefix secrets with `prod-`
- Use separate service account with restricted permissions
- Enable audit logging

## Security Best Practices

1. Secret Rotation
   - Rotate database credentials every 90 days
   - Rotate API keys according to provider recommendations
   - Use automated rotation where possible

2. Access Control
   - Limit secret access to necessary services only
   - Use separate service accounts per environment
   - Regularly audit access logs

3. Monitoring
   - Set up alerts for secret access
   - Monitor for unauthorized access attempts
   - Track secret version changes

4. CI/CD Security
   - Use encrypted secrets in GitHub Actions
   - Never log secret values
   - Rotate CI/CD service account keys regularly

## Troubleshooting

1. Secret Access Issues
   - Verify service account permissions
   - Check secret versions
   - Validate secret names and paths

2. CI/CD Failures
   - Verify GitHub secrets are set correctly
   - Check service account key validity
   - Verify secret mounting in containers

## Migration from .env

1. Backup current values:
```bash
cp .env .env.backup
```

2. Move values to GitHub Secrets:
```bash
# Use GitHub UI or API to set secrets
```

3. Move values to Google Cloud:
```bash
# Use the commands in the "Google Cloud Secret Manager Setup" section
```

4. Update application code to use secrets:
```rust
// Instead of
let database_url = env::var("DATABASE_URL")?;

// Use
let database_url = gcp_secrets::get_secret("DATABASE_URL")?;
```

5. Remove sensitive values from .env:
```bash
# Keep only non-sensitive configuration
RUST_LOG=debug
PORT=3000
HOST=127.0.0.1
ENVIRONMENT=development
```
