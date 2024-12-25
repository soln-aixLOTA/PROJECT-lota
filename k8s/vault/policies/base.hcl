# Base policy for all LotaBots services
# This policy contains common permissions needed by all services

# Allow token renewal
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Allow lease management
path "sys/leases/renew" {
  capabilities = ["update"]
}

path "sys/renew/*" {
  capabilities = ["update"]
}

# Allow access to shared secrets
path "lotabots/shared/*" {
  capabilities = ["read"]
}

# Allow health checks
path "sys/health" {
  capabilities = ["read"]
}

# Allow service to look up its own token
path "auth/token/lookup-self" {
  capabilities = ["read"]
}

# Allow service to revoke its own token on shutdown
path "auth/token/revoke-self" {
  capabilities = ["update"]
}

# Allow metrics collection
path "sys/metrics" {
  capabilities = ["read"]
}

# Allow service to manage its own leases
path "sys/leases/lookup" {
  capabilities = ["update"]
}

path "sys/leases/revoke" {
  capabilities = ["update"]
} 