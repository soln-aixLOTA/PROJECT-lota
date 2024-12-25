from typing import Optional
from pydantic import BaseSettings
from src.lib.secrets.manager import SecretsManager

class InferenceServiceConfig(BaseSettings):
    """Configuration for the Inference Service"""
    
    # Service configuration
    service_name: str = "inference-service"
    service_port: int = 50051
    
    # NVIDIA configuration
    nvidia_api_key: Optional[str] = None
    triton_url: str = "localhost:8001"
    
    # Model configuration
    model_repository_path: str = "/models"
    default_model_name: str = "gemini-2.0-flash-exp"
    
    # Metrics configuration
    enable_metrics: bool = True
    metrics_port: int = 8080
    
    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._load_secrets()
    
    def _load_secrets(self):
        """Load secrets from Vault"""
        try:
            secrets_manager = SecretsManager.get_instance()
            
            # Load NVIDIA API key
            self.nvidia_api_key = secrets_manager.get_secret(
                path="nvidia/api",
                key="NVIDIA_API_KEY"
            )
            
        except Exception as e:
            raise Exception(f"Failed to load secrets from Vault: {str(e)}")
    
    class Config:
        env_prefix = "INFERENCE_"
        case_sensitive = False 