#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Setting up secrets for LOTA AI...${NC}"

# Check if gh CLI is installed
if ! command -v gh &>/dev/null; then
    echo -e "${RED}GitHub CLI (gh) is not installed. Please install it first:${NC}"
    echo "https://cli.github.com/manual/installation"
    exit 1
fi

# Check if gcloud CLI is installed
if ! command -v gcloud &>/dev/null; then
    echo -e "${RED}Google Cloud CLI (gcloud) is not installed. Please install it first:${NC}"
    echo "https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Ensure user is logged in to GitHub
if ! gh auth status &>/dev/null; then
    echo -e "${RED}Please login to GitHub first:${NC}"
    gh auth login
fi

# Set up Google Cloud authentication
echo -e "${BLUE}Setting up Google Cloud authentication...${NC}"
SERVICE_ACCOUNT_KEY="cs-hc-cd812496a16440caaa406d73-e61fd97e4b34.json"

if [[ ! -f "$SERVICE_ACCOUNT_KEY" ]]; then
    echo -e "${RED}Service account key file not found: $SERVICE_ACCOUNT_KEY${NC}"
    exit 1
fi

# Authenticate with Google Cloud using service account
echo -e "${BLUE}Authenticating with Google Cloud...${NC}"
gcloud auth activate-service-account --key-file="$SERVICE_ACCOUNT_KEY"

# Set the project ID
PROJECT_ID="cs-hc-cd812496a16440caaa406d73"
gcloud config set project "$PROJECT_ID"

echo -e "${BLUE}Setting up GitHub Secrets...${NC}"

# Function to set GitHub secret
set_github_secret() {
    local name=$1
    local value=$2
    echo -e "${BLUE}Setting GitHub secret: ${name}${NC}"
    echo -n "$value" | gh secret set "$name"
}

# Database secrets
read -p "Enter PostgreSQL username: " POSTGRES_USER
read -s -p "Enter PostgreSQL password: " POSTGRES_PASSWORD
echo
read -p "Enter PostgreSQL database name: " POSTGRES_DB
read -p "Enter complete DATABASE_URL (or press enter to generate): " DATABASE_URL
if [ -z "$DATABASE_URL" ]; then
    DATABASE_URL="postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB}"
fi

# JWT secret
JWT_SECRET=$(openssl rand -base64 32)

# API Keys
read -p "Enter Hugging Face Hub Token: " HUGGING_FACE_HUB_TOKEN
read -p "Enter OpenAI API Key: " OPENAI_API_KEY

# Google Cloud Configuration
GCP_PROJECT="$PROJECT_ID"
read -p "Enter GKE Cluster name: " GKE_CLUSTER
read -p "Enter GKE Zone: " GKE_ZONE

# Set GitHub secrets
echo -e "${BLUE}Setting GitHub secrets...${NC}"
set_github_secret "POSTGRES_USER" "$POSTGRES_USER"
set_github_secret "POSTGRES_PASSWORD" "$POSTGRES_PASSWORD"
set_github_secret "POSTGRES_DB" "$POSTGRES_DB"
set_github_secret "DATABASE_URL" "$DATABASE_URL"
set_github_secret "JWT_SECRET" "$JWT_SECRET"
set_github_secret "HUGGING_FACE_HUB_TOKEN" "$HUGGING_FACE_HUB_TOKEN"
set_github_secret "OPENAI_API_KEY" "$OPENAI_API_KEY"
set_github_secret "GCP_PROJECT" "$GCP_PROJECT"
set_github_secret "GCP_SA_KEY" "$(cat $SERVICE_ACCOUNT_KEY)"
set_github_secret "GKE_CLUSTER" "$GKE_CLUSTER"
set_github_secret "GKE_ZONE" "$GKE_ZONE"

echo -e "${BLUE}Setting up Google Cloud Secrets...${NC}"

# Enable Secret Manager API
gcloud services enable secretmanager.googleapis.com

# Function to set Google Cloud secret
set_gcp_secret() {
    local name=$1
    local value=$2
    echo -e "${BLUE}Setting Google Cloud secret: ${name}${NC}"

    # Create secret if it doesn't exist
    if ! gcloud secrets describe "$name" &>/dev/null; then
        gcloud secrets create "$name" --replication-policy="automatic"
    fi

    # Add new version
    echo -n "$value" | gcloud secrets versions add "$name" --data-file=-
}

# Set Google Cloud secrets
set_gcp_secret "DATABASE_URL" "$DATABASE_URL"
set_gcp_secret "JWT_SECRET" "$JWT_SECRET"
set_gcp_secret "HUGGING_FACE_HUB_TOKEN" "$HUGGING_FACE_HUB_TOKEN"
set_gcp_secret "OPENAI_API_KEY" "$OPENAI_API_KEY"

echo -e "${GREEN}✅ Secrets setup completed successfully!${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo "1. Update your .env file to use non-sensitive values only"
echo "2. Update your application code to use Google Cloud Secret Manager"
echo "3. Test your CI/CD pipeline"
echo "4. Remove any hardcoded secrets from your codebase"
