# Include base policy
path "sys/policies/acl/base" {
  capabilities = ["read"]
}

# API Gateway specific permissions
path "lotabots/api-gateway/*" {
  capabilities = ["read"]
}

# Allow API Gateway to validate other services' tokens
path "auth/token/lookup" {
  capabilities = ["update"]
}

# Allow API Gateway to handle rate limiting data
path "lotabots/rate-limit/*" {
  capabilities = ["create", "read", "update", "delete"]
}

# Allow API Gateway to access usage tracking data
path "lotabots/usage/*" {
  capabilities = ["create", "read", "update"]
} 