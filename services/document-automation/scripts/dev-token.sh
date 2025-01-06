#!/bin/bash

# Script to generate development JWT tokens for the Document Automation Service

# Default values
SECRET="dev-secret-change-me-in-production"
EXPIRATION_HOURS=24
USER_ID="dev-user"
ROLE="admin"

# Help message
show_help() {
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  -s, --secret SECRET       JWT secret (default: dev-secret)"
    echo "  -e, --expiry HOURS        Token expiration in hours (default: 24)"
    echo "  -u, --user USER_ID        User ID (default: dev-user)"
    echo "  -r, --role ROLE           User role (default: admin)"
    echo "  -h, --help               Show this help message"
    echo
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -s|--secret)
            SECRET="$2"
            shift 2
            ;;
        -e|--expiry)
            EXPIRATION_HOURS="$2"
            shift 2
            ;;
        -u|--user)
            USER_ID="$2"
            shift 2
            ;;
        -r|--role)
            ROLE="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Calculate expiration timestamp
EXPIRATION=$(date -u -d "+${EXPIRATION_HOURS} hours" +%s)

# Create JWT header
HEADER='{
    "alg": "HS256",
    "typ": "JWT"
}'

# Create JWT payload
PAYLOAD='{
    "sub": "'$USER_ID'",
    "role": "'$ROLE'",
    "exp": '$EXPIRATION'
}'

# Base64 encode header and payload
HEADER_BASE64=$(echo -n "$HEADER" | base64 | tr -d '=' | tr '/+' '_-')
PAYLOAD_BASE64=$(echo -n "$PAYLOAD" | base64 | tr -d '=' | tr '/+' '_-')

# Create signature
SIGNATURE=$(echo -n "${HEADER_BASE64}.${PAYLOAD_BASE64}" | \
    openssl dgst -binary -sha256 -hmac "$SECRET" | \
    base64 | tr -d '=' | tr '/+' '_-')

# Combine to create JWT
JWT="${HEADER_BASE64}.${PAYLOAD_BASE64}.${SIGNATURE}"

# Output token
echo "$JWT"

# Optional: decode and verify
if command -v jwt &> /dev/null; then
    echo -e "\nDecoded token:"
    echo "$JWT" | jwt decode -
    
    echo -e "\nVerifying token:"
    echo "$JWT" | jwt verify --secret "$SECRET"
else
    echo -e "\nInstall 'jwt-cli' for token verification:"
    echo "cargo install jwt-cli"
fi 