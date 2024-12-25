import os
from typing import Optional, Dict, Any
import hvac
from functools import lru_cache

class SecretsManager:
    def __init__(self):
        vault_addr = os.getenv('VAULT_ADDR', 'http://vault.lotabots.svc:8200')
        vault_token = os.getenv('VAULT_TOKEN')
        
        if not vault_token:
            raise ValueError("VAULT_TOKEN environment variable must be set")
        
        self.client = hvac.Client(
            url=vault_addr,
            token=vault_token
        )
        
        if not self.client.is_authenticated():
            raise Exception("Failed to authenticate with Vault")
    
    @lru_cache(maxsize=100)
    def get_secret(self, path: str, key: str) -> str:
        """
        Get a secret from Vault. Results are cached to reduce Vault requests.
        
        Args:
            path: The path to the secret in Vault
            key: The key within the secret
            
        Returns:
            The secret value
            
        Raises:
            Exception: If the secret cannot be retrieved
        """
        try:
            secret = self.client.secrets.kv.v2.read_secret_version(
                path=path,
                mount_point='secret'
            )
            
            if not secret or 'data' not in secret or 'data' not in secret['data']:
                raise Exception(f"Secret not found at path: {path}")
                
            value = secret['data']['data'].get(key)
            if value is None:
                raise Exception(f"Key '{key}' not found in secret at path: {path}")
                
            return value
            
        except Exception as e:
            raise Exception(f"Failed to get secret: {str(e)}")
    
    def set_secret(self, path: str, data: Dict[str, Any]) -> None:
        """
        Set a secret in Vault
        
        Args:
            path: The path where to store the secret
            data: Dictionary containing the secret data
            
        Raises:
            Exception: If the secret cannot be set
        """
        try:
            self.client.secrets.kv.v2.create_or_update_secret(
                path=path,
                secret=data,
                mount_point='secret'
            )
            
            # Clear the cache for this path
            self.get_secret.cache_clear()
            
        except Exception as e:
            raise Exception(f"Failed to set secret: {str(e)}")
    
    @staticmethod
    @lru_cache(maxsize=1)
    def get_instance() -> 'SecretsManager':
        """
        Get or create a singleton instance of SecretsManager
        
        Returns:
            SecretsManager instance
        """
        return SecretsManager() 