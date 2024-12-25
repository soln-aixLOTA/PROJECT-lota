# Include base policy
path "sys/policies/acl/base" {
  capabilities = ["read"]
}

# User Authentication service specific permissions
path "lotabots/user-auth/*" {
  capabilities = ["read"]
}

# Allow access to database credentials
path "lotabots/database/*" {
  capabilities = ["read"]
}

# Allow access to user authentication data
path "lotabots/auth/*" {
  capabilities = ["create", "read", "update", "delete"]
}

# Allow access to token management
path "lotabots/tokens/*" {
  capabilities = ["create", "read", "update", "delete"]
}

# Allow access to user session data
path "lotabots/sessions/*" {
  capabilities = ["create", "read", "update", "delete"]
} 