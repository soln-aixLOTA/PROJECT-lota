#!/bin/bash

# Source common functions
source ./lib/common.sh

# Check if JWT_SECRET is set
if [ -z "${JWT_SECRET}" ]; then
    echo "❌ Error: JWT_SECRET environment variable is not set"
    echo "Please set it securely using one of these methods:"
    echo "1. Generate and export directly:"
    echo "   export JWT_SECRET=\$(openssl rand -hex 32)"
    echo "2. Add to your .env file (ensure it's in .gitignore):"
    echo "   echo \"JWT_SECRET=\$(openssl rand -hex 32)\" >> .env"
    exit 1
fi

# Use the environment variable
SECRET="${JWT_SECRET}"
EXPIRATION="${JWT_EXPIRATION:-3600}"  # 1 hour default
USER_ID="test-user"
ROLE="user"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --expiration)
            EXPIRATION="$2"
            shift 2
            ;;
        --user)
            USER_ID="$2"
            shift 2
            ;;
        --role)
            ROLE="$2"
            shift 2
            ;;
        --key)
            KEY="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [options]"
            echo
            echo "Options:"
            echo "  --key KEY          Key for token signing (default: from JWT_SECRET environment variable)"
            echo "  --expiration SECS  Token expiration in seconds (default: from environment or 3600)"
            echo "  --user ID          User ID to include in token (default: test-user)"
            echo "  --role ROLE        User role to include in token (default: user)"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Generate token using jwt-cli if available, otherwise use Python
if command -v jwt &> /dev/null; then
    TOKEN=$(jwt encode --secret "$KEY" --alg HS256 --exp "+${EXPIRATION}s" --claim "sub=$USER_ID" --claim "role=$ROLE")
else
    # Use Python as fallback
    TOKEN=$(python3 -c "
import jwt
import time

payload = {
    'sub': '$USER_ID',
    'role': '$ROLE',
    'exp': int(time.time()) + $EXPIRATION
}

token = jwt.encode(payload, '$KEY', algorithm='HS256')
if isinstance(token, bytes):
    token = token.decode('utf-8')
print(token, end='')
")
fi

if [ -z "$TOKEN" ]; then
    log_error "Failed to generate token"
    exit 1
fi

echo "$TOKEN"
