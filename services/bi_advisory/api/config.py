from pydantic_settings import BaseSettings
from functools import lru_cache
from typing import Optional
import os

class Settings(BaseSettings):
    """Application settings"""

    # API Configuration
    API_V1_STR: str = "/api/v1"
    PROJECT_NAME: str = "BI and AI Advisory System"
    VERSION: str = "1.0.0"
    DEBUG: bool = False

    # Database Configuration
    DATABASE_URL: str = os.getenv(
        "DATABASE_URL",
        "postgresql://postgres:postgres@localhost:5432/bi_advisory"
    )

    # Security
    SECRET_KEY: str = os.getenv("SECRET_KEY", "your-secret-key-here")
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 60 * 24 * 8  # 8 days

    # CORS Configuration
    BACKEND_CORS_ORIGINS: list = ["*"]  # In production, replace with specific origins

    # Cache Configuration
    REDIS_URL: Optional[str] = os.getenv("REDIS_URL", "redis://localhost:6379/0")
    CACHE_TTL: int = 300  # 5 minutes

    # ML Model Configuration
    MODEL_PATH: str = os.getenv("MODEL_PATH", "models")
    INFERENCE_TIMEOUT: int = 30  # seconds

    # Logging Configuration
    LOG_LEVEL: str = "INFO"
    LOG_FORMAT: str = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"

    # Performance Configuration
    MAX_WORKERS: int = 4
    CHUNK_SIZE: int = 1000

    # Feature Flags
    ENABLE_ML_FEATURES: bool = True
    ENABLE_CACHING: bool = True
    ENABLE_ANALYTICS: bool = True

    class Config:
        case_sensitive = True
        env_file = ".env"

@lru_cache()
def get_settings() -> Settings:
    """Get cached settings"""
    return Settings()

# Initialize settings
settings = get_settings()
