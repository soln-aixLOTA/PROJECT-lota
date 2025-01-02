#!/bin/bash

set -e

echo "AWS Credentials Setup"
echo "===================="
echo
echo "This script will help you configure AWS credentials securely."
echo "Please ensure you have rotated your AWS credentials before proceeding."
echo

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    echo "AWS CLI is not installed. Please install it first."
    echo "Visit: https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"
    exit 1
fi

# Create AWS credentials directory if it doesn't exist
mkdir -p ~/.aws

# Check if credentials file already exists
if [ -f ~/.aws/credentials ]; then
    echo "AWS credentials file already exists."
    echo "Would you like to update it? (y/n)"
    read -r update_credentials
    if [ "$update_credentials" != "y" ]; then
        echo "Skipping credentials update."
        exit 0
    fi
fi

# Get AWS credentials
echo
echo "Please enter your AWS credentials:"
echo "Note: These should be from a dedicated IAM user with appropriate permissions."
echo
read -p "AWS Access Key ID: " aws_access_key_id
read -p "AWS Secret Access Key: " aws_secret_access_key
read -p "Default region [us-east-1]: " aws_region
aws_region=${aws_region:-us-east-1}

# Write to credentials file
cat > ~/.aws/credentials << EOF
[default]
aws_access_key_id = ${aws_access_key_id}
aws_secret_access_key = ${aws_secret_access_key}
region = ${aws_region}
EOF

# Set proper permissions
chmod 600 ~/.aws/credentials

echo
echo "AWS credentials have been configured successfully!"
echo "Credentials file: ~/.aws/credentials"
echo
echo "Next steps:"
echo "1. Verify the configuration: aws sts get-caller-identity"
echo "2. Run the Terraform initialization script: ./init.sh <environment>" 