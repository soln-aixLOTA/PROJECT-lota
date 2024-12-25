use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Environment variable error: {0}")]
    EnvError(#[from] env::VarError),
    
    #[error("Configuration error: {0}")]
    Other(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VaultConfig {
    pub addr: String,
    pub role: String,
    pub namespace: String,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            addr: "http://vault.lotabots.svc:8200".to_string(),
            role: "default".to_string(),
            namespace: "lotabots".to_string(),
        }
    }
}

impl VaultConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            addr: env::var("VAULT_ADDR")
                .unwrap_or_else(|_| "http://vault.lotabots.svc:8200".to_string()),
            role: env::var("VAULT_ROLE")?,
            namespace: env::var("VAULT_NAMESPACE")
                .unwrap_or_else(|_| "lotabots".to_string()),
        })
    }
    
    pub fn with_role(mut self, role: &str) -> Self {
        self.role = role.to_string();
        self
    }
    
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = namespace.to_string();
        self
    }
    
    pub fn with_addr(mut self, addr: &str) -> Self {
        self.addr = addr.to_string();
        self
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub prefix: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "vault".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ttl_seconds: 3600,
            max_size: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretsConfig {
    pub vault: VaultConfig,
    pub metrics: MetricsConfig,
    pub cache: CacheConfig,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            vault: VaultConfig::default(),
            metrics: MetricsConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl SecretsConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            vault: VaultConfig::from_env()?,
            metrics: MetricsConfig::default(),
            cache: CacheConfig::default(),
        })
    }
    
    pub fn with_vault_config(mut self, vault: VaultConfig) -> Self {
        self.vault = vault;
        self
    }
    
    pub fn with_metrics_config(mut self, metrics: MetricsConfig) -> Self {
        self.metrics = metrics;
        self
    }
    
    pub fn with_cache_config(mut self, cache: CacheConfig) -> Self {
        self.cache = cache;
        self
    }
} 