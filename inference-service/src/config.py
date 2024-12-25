import os
import logging
from typing import Optional, Dict, Any
import hvac
import cachetools
import asyncio
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)

class SecretsManager:
    def __init__(self):
        self.client = hvac.Client(
            url=os.getenv("VAULT_ADDR", "http://vault.lotabots.svc:8200")
        )
        self.namespace = os.getenv("VAULT_NAMESPACE", "lotabots")
        self.role = os.getenv("VAULT_ROLE", "inference-service")
        self._cache = cachetools.TTLCache(maxsize=100, ttl=3600)  # 1 hour cache
        self._lease_renewal_task: Optional[asyncio.Task] = None
        self._leases: Dict[str, datetime] = {}
        
    async def init(self):
        """Initialize the Vault client with Kubernetes authentication."""
        try:
            # Read the Kubernetes service account token
            with open("/var/run/secrets/kubernetes.io/serviceaccount/token") as f:
                jwt = f.read()
            
            # Authenticate with Vault using Kubernetes auth method
            auth_resp = self.client.auth.kubernetes.login(
                role=self.role,
                jwt=jwt
            )
            
            if not auth_resp.get('auth'):
                raise Exception("Failed to authenticate with Vault")
                
            logger.info("Successfully authenticated with Vault")
            
            # Start lease renewal task
            self._lease_renewal_task = asyncio.create_task(self._renew_leases())
            
        except Exception as e:
            logger.error(f"Failed to initialize Vault client: {e}")
            raise
            
    async def get_secret(self, path: str) -> str:
        """Get a secret from Vault with caching."""
        # Check cache first
        cache_key = f"{self.namespace}/{path}"
        if cache_key in self._cache:
            return self._cache[cache_key]
            
        try:
            # Read from Vault
            secret = self.client.secrets.kv.v2.read_secret_version(
                path=f"{self.namespace}/{path}"
            )
            
            if not secret or 'data' not in secret or 'data' not in secret['data']:
                raise Exception(f"Secret not found at path: {path}")
                
            value = secret['data']['data']
            
            # Cache the result
            self._cache[cache_key] = value
            
            # If there's a lease, register it for renewal
            if 'lease_id' in secret:
                self._leases[secret['lease_id']] = datetime.now() + \
                    timedelta(seconds=secret['lease_duration'])
                
            return value
            
        except Exception as e:
            logger.error(f"Failed to read secret at path {path}: {e}")
            raise
            
    async def get_dynamic_secret(self, path: str) -> Dict[str, Any]:
        """Get a dynamic secret from Vault (no caching)."""
        try:
            secret = self.client.secrets.kv.v2.read_secret_version(
                path=f"{self.namespace}/{path}"
            )
            
            if not secret or 'data' not in secret or 'data' not in secret['data']:
                raise Exception(f"Dynamic secret not found at path: {path}")
                
            # Register lease for renewal
            if 'lease_id' in secret:
                self._leases[secret['lease_id']] = datetime.now() + \
                    timedelta(seconds=secret['lease_duration'])
                
            return secret['data']['data']
            
        except Exception as e:
            logger.error(f"Failed to read dynamic secret at path {path}: {e}")
            raise
            
    async def _renew_leases(self):
        """Background task to renew leases."""
        while True:
            try:
                current_time = datetime.now()
                for lease_id, expiry in list(self._leases.items()):
                    # Renew lease if it's within 10 minutes of expiring
                    if current_time + timedelta(minutes=10) >= expiry:
                        try:
                            self.client.sys.renew_lease(
                                lease_id=lease_id
                            )
                            logger.debug(f"Successfully renewed lease: {lease_id}")
                        except Exception as e:
                            logger.error(f"Failed to renew lease {lease_id}: {e}")
                            del self._leases[lease_id]
                            
            except Exception as e:
                logger.error(f"Error in lease renewal task: {e}")
                
            await asyncio.sleep(60)  # Check every minute
            
    async def cleanup(self):
        """Cleanup resources."""
        if self._lease_renewal_task:
            self._lease_renewal_task.cancel()
            try:
                await self._lease_renewal_task
            except asyncio.CancelledError:
                pass 