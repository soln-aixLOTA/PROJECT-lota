#!/bin/bash

# Source common functions
source ./lib/common.sh

# Default values from environment if set
SECRET="${JWT_SECRET:-development_secret_key}"
EXPIRATION="${JWT_EXPIRATION:-3600}"  # 1 hour
USER_ID="test-user"
ROLE="user"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --secret)
            SECRET="$2"
            shift 2
            ;;
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
        -h|--help)
            echo "Usage: $0 [options]"
            echo
            echo "Options:"
            echo "  --secret KEY       Secret key for token signing (default: from environment or development_secret_key)"
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
    TOKEN=$(jwt encode --secret "$SECRET" --alg HS256 --exp "+${EXPIRATION}s" --claim "sub=$USER_ID" --claim "role=$ROLE")
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

token = jwt.encode(payload, '$SECRET', algorithm='HS256')
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