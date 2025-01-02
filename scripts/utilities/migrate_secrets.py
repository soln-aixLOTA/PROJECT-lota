#!/usr/bin/env python3

import os
import sys
import yaml
import hvac
from typing import Dict, Any

def load_secrets_from_env() -> Dict[str, Any]:
    """Load secrets from environment files"""
    secrets = {}
    env_files = ['.env', '.env.local', '.env.development', '.env.production']
    
    for env_file in env_files:
        if os.path.exists(env_file):
            with open(env_file, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith('#'):
                        try:
                            key, value = line.split('=', 1)
                            secrets[key.strip()] = value.strip()
                        except ValueError:
                            print(f"Warning: Skipping malformed line in {env_file}: {line}")
    
    return secrets

def get_vault_client() -> hvac.Client:
    """Initialize and return an authenticated Vault client"""
    vault_addr = os.getenv('VAULT_ADDR', 'http://vault.lotabots.svc:8200')
    vault_token = os.getenv('VAULT_TOKEN')
    
    if not vault_token:
        raise ValueError("VAULT_TOKEN environment variable must be set")
    
    client = hvac.Client(
        url=vault_addr,
        token=vault_token
    )
    
    if not client.is_authenticated():
        raise Exception("Failed to authenticate with Vault")
    
    return client

def migrate_secrets_to_vault(client: hvac.Client, secrets: Dict[str, Any]) -> None:
    """Migrate secrets to Vault with appropriate paths"""
    # Define secret paths for different types of secrets
    secret_paths = {
        'JWT_': 'auth/jwt',
        'DB_': 'database/credentials',
        'NVIDIA_': 'nvidia/api',
        'AWS_': 'cloud/aws',
        'AZURE_': 'cloud/azure',
        'GOOGLE_': 'cloud/google',
    }
    
    # Group secrets by their type
    grouped_secrets: Dict[str, Dict[str, str]] = {}
    
    for key, value in secrets.items():
        path = 'misc'  # default path
        for prefix, secret_path in secret_paths.items():
            if key.startswith(prefix):
                path = secret_path
                break
        
        if path not in grouped_secrets:
            grouped_secrets[path] = {}
        grouped_secrets[path][key] = value
    
    # Write secrets to Vault
    for path, secret_data in grouped_secrets.items():
        try:
            client.secrets.kv.v2.create_or_update_secret(
                path=path,
                secret=secret_data,
                mount_point='secret'
            )
            print(f"Successfully migrated secrets to path: secret/{path}")
        except Exception as e:
            print(f"Error migrating secrets to path {path}: {str(e)}")

def main():
    try:
        # Load secrets from environment files
        secrets = load_secrets_from_env()
        if not secrets:
            print("No secrets found in environment files")
            return
        
        # Get Vault client
        client = get_vault_client()
        
        # Migrate secrets to Vault
        migrate_secrets_to_vault(client, secrets)
        
        print("\nSecret migration completed successfully!")
        
    except Exception as e:
        print(f"Error during secret migration: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == '__main__':
    main() 