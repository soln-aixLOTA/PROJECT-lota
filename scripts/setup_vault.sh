#!/bin/bash

set -e

# Create namespace if it doesn't exist
kubectl create namespace lotabots --dry-run=client -o yaml | kubectl apply -f -

# Deploy Vault
echo "Deploying Vault..."
kubectl apply -f k8s/base/vault/vault-config.yaml
kubectl apply -f k8s/base/vault/vault-statefulset.yaml

# Wait for Vault pods to be ready
echo "Waiting for Vault pods to be ready..."
kubectl wait --for=condition=ready pod -l app=vault -n lotabots --timeout=300s

# Initialize Vault on the first pod
echo "Initializing Vault..."
INIT_RESPONSE=$(kubectl exec -n lotabots vault-0 -- vault operator init -format=json)
UNSEAL_KEYS=$(echo $INIT_RESPONSE | jq -r '.unseal_keys_b64[]')
ROOT_TOKEN=$(echo $INIT_RESPONSE | jq -r '.root_token')

# Save keys and token securely
echo "Saving Vault credentials..."
mkdir -p ~/.vault-keys
echo $INIT_RESPONSE > ~/.vault-keys/vault-keys.json
chmod 600 ~/.vault-keys/vault-keys.json

# Unseal Vault on each pod
for pod in vault-0 vault-1 vault-2; do
    echo "Unsealing Vault on $pod..."
    for key in $UNSEAL_KEYS; do
        kubectl exec -n lotabots $pod -- vault operator unseal $key
    done
done

# Configure Vault
echo "Configuring Vault..."
export VAULT_TOKEN=$ROOT_TOKEN
export VAULT_ADDR=http://localhost:8200

# Port forward Vault service
kubectl port-forward -n lotabots svc/vault 8200:8200 &
PF_PID=$!
sleep 5

# Enable secrets engines
vault secrets enable -path=secret kv-v2

# Enable Kubernetes authentication
vault auth enable kubernetes

# Configure Kubernetes authentication
KUBE_HOST=$(kubectl config view --raw --minify --flatten --output='jsonpath={.clusters[].cluster.server}')
KUBE_CA_CERT=$(kubectl config view --raw --minify --flatten --output='jsonpath={.clusters[].cluster.certificate-authority-data}' | base64 --decode)
KUBE_TOKEN=$(kubectl create token vault-auth)

vault write auth/kubernetes/config \
    kubernetes_host="$KUBE_HOST" \
    kubernetes_ca_cert="$KUBE_CA_CERT" \
    token_reviewer_jwt="$KUBE_TOKEN"

# Create policies for services
cat > /tmp/api-gateway-policy.hcl << EOF
path "secret/data/auth/jwt/*" {
  capabilities = ["read"]
}
EOF

cat > /tmp/inference-service-policy.hcl << EOF
path "secret/data/nvidia/api/*" {
  capabilities = ["read"]
}
EOF

vault policy write api-gateway /tmp/api-gateway-policy.hcl
vault policy write inference-service /tmp/inference-service-policy.hcl

# Create Kubernetes service account roles
vault write auth/kubernetes/role/api-gateway \
    bound_service_account_names=api-gateway \
    bound_service_account_namespaces=lotabots \
    policies=api-gateway \
    ttl=1h

vault write auth/kubernetes/role/inference-service \
    bound_service_account_names=inference-service \
    bound_service_account_namespaces=lotabots \
    policies=inference-service \
    ttl=1h

# Migrate existing secrets
echo "Migrating secrets..."
python3 scripts/migrate_secrets.py

# Clean up
kill $PF_PID

echo "Vault setup completed successfully!"
echo "Root token is saved in ~/.vault-keys/vault-keys.json"
echo "Please store these credentials securely and remove them from the filesystem after backing up" 