# Include base policy
path "sys/policies/acl/base" {
  capabilities = ["read"]
}

# Inference service specific permissions
path "lotabots/inference-service/*" {
  capabilities = ["read"]
}

# Allow access to NVIDIA API keys and credentials
path "lotabots/nvidia/*" {
  capabilities = ["read"]
}

# Allow access to model credentials
path "lotabots/models/*" {
  capabilities = ["read"]
}

# Allow access to GPU metrics data
path "lotabots/metrics/gpu/*" {
  capabilities = ["create", "read", "update"]
} 