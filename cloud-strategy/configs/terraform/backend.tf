# Backend infrastructure for Terraform state management

provider "aws" {
  region = var.aws_region
  alias  = "backend"

  default_tags {
    tags = {
      Project     = "${var.project_name}-backend"
      ManagedBy   = "terraform"
      Environment = var.environment
    }
  }
}

# S3 bucket for Terraform state
resource "aws_s3_bucket" "terraform_state" {
  provider = aws.backend
  bucket   = "${var.project_name}-terraform-state-${var.environment}"

  # Prevent accidental deletion of this S3 bucket
  lifecycle {
    prevent_destroy = true
  }
}

# Enable versioning for state files
resource "aws_s3_bucket_versioning" "terraform_state" {
  provider = aws.backend
  bucket   = aws_s3_bucket.terraform_state.id
  versioning_configuration {
    status = "Enabled"
  }
}

# Enable server-side encryption by default
resource "aws_s3_bucket_server_side_encryption_configuration" "terraform_state" {
  provider = aws.backend
  bucket   = aws_s3_bucket.terraform_state.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# Block all public access to the S3 bucket
resource "aws_s3_bucket_public_access_block" "terraform_state" {
  provider = aws.backend
  bucket   = aws_s3_bucket.terraform_state.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# DynamoDB table for state locking
resource "aws_dynamodb_table" "terraform_locks" {
  provider     = aws.backend
  name         = "${var.project_name}-terraform-locks-${var.environment}"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "LockID"

  attribute {
    name = "LockID"
    type = "S"
  }
}

# Outputs for backend configuration
output "terraform_state_bucket" {
  value       = aws_s3_bucket.terraform_state.bucket
  description = "S3 bucket for Terraform state storage"
}

output "terraform_locks_table" {
  value       = aws_dynamodb_table.terraform_locks.name
  description = "DynamoDB table for Terraform state locking"
}
