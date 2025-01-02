#!/bin/bash

set -e

# Check if environment is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <environment>"
    echo "Example: $0 dev"
    exit 1
fi

ENVIRONMENT=$1
REGION=${2:-us-east-1}  # Default to us-east-1 if not specified

echo "Initializing Terraform backend infrastructure for environment: $ENVIRONMENT"

# Initialize and apply backend configuration first
terraform init
terraform apply -target=aws_s3_bucket.terraform_state \
               -target=aws_s3_bucket_versioning.terraform_state \
               -target=aws_s3_bucket_server_side_encryption_configuration.terraform_state \
               -target=aws_s3_bucket_public_access_block.terraform_state \
               -target=aws_dynamodb_table.terraform_locks \
               -var="environment=$ENVIRONMENT" \
               -var="aws_region=$REGION"

# Get the backend values
BUCKET_NAME=$(terraform output -raw terraform_state_bucket)
TABLE_NAME=$(terraform output -raw terraform_locks_table)

echo "Backend infrastructure created successfully:"
echo "S3 Bucket: $BUCKET_NAME"
echo "DynamoDB Table: $TABLE_NAME"

# Update backend configuration in main.tf
echo "Updating backend configuration in main.tf..."
sed -i "s/bucket.*=.*\".*\"/bucket = \"$BUCKET_NAME\"/" main.tf
sed -i "s/dynamodb_table.*=.*\".*\"/dynamodb_table = \"$TABLE_NAME\"/" main.tf
sed -i "s/region.*=.*\".*\"/region = \"$REGION\"/" main.tf

echo "Initializing main Terraform configuration..."
terraform init -reconfigure

echo "Ready to apply main infrastructure. Run 'terraform plan' to review changes." 