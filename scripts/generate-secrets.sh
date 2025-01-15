#!/bin/bash

# Function to generate a secure secret using OpenSSL
generate_secret() {
    openssl rand -hex 32
}

# Function to update .env file
update_env() {
    local env_file=".env"
    local secret_name="$1"
    local secret_value="$2"

    # Create .env file if it doesn't exist
    touch "$env_file"

    # Remove existing line if present
    sed -i "/$secret_name=/d" "$env_file"

    # Add new secret
    echo "$secret_name=$secret_value" >>"$env_file"
}

# Main script
echo "Generating secure secrets for LotaBots API Gateway..."

# Generate JWT secret if not already set
if [ -z "$JWT_SECRET" ]; then
    JWT_SECRET=$(generate_secret)
    echo "Generated new JWT_SECRET"
    update_env "JWT_SECRET" "$JWT_SECRET"
else
    echo "JWT_SECRET is already set in environment"
fi

echo "
Secrets have been generated and saved to .env file.
Make sure to:
1. Never commit the .env file to version control
2. Keep your secrets secure and rotate them periodically
3. Use different secrets for development, staging, and production environments

To use in development:
   source .env

To use in production:
   - Use a secure secrets management service
   - Set environment variables securely in your deployment platform
"
