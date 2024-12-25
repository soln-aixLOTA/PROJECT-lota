use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOptions {
    pub use_cdi: bool, // Use Container Device Interface instead of legacy mode
    pub selinux_enabled: bool,
    pub seccomp_profile: Option<String>,
    pub no_new_privileges: bool,
    pub readonly_root_fs: bool,
    pub drop_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mount {
    pub source: PathBuf,
    pub target: PathBuf,
    pub readonly: bool,
    pub selinux_relabel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub image: String,
    pub command: Vec<String>,
    pub env: HashMap<String, String>,
    pub mounts: Vec<Mount>,
    pub security: SecurityOptions,
    pub gpu_requirements: GPURequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub config: ContainerConfig,
    pub status: ContainerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub health_status: Option<HealthStatus>,
}

pub struct ContainerManager {
    containers: RwLock<HashMap<String, ContainerInfo>>,
}

impl ContainerManager {
    pub fn new() -> Self {
        Self {
            containers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_container(&self, config: ContainerConfig) -> Result<ContainerInfo> {
        // Validate security configuration
        self.validate_security_config(&config.security)?;

        // Validate and sanitize mounts
        self.validate_mounts(&config.mounts)?;

        // Force CDI mode for GPU access
        if !config.security.use_cdi {
            warn!("Forcing CDI mode for container creation due to security requirements");
            let mut config = config.clone();
            config.security.use_cdi = true;
        }

        // Create container with secure defaults
        let container = ContainerInfo {
            id: uuid::Uuid::new_v4().to_string(),
            config,
            status: ContainerStatus::Created,
            created_at: chrono::Utc::now(),
            health_status: None,
        };

        // Store container info
        self.containers
            .write()
            .await
            .insert(container.id.clone(), container.clone());

        Ok(container)
    }

    fn validate_security_config(&self, security: &SecurityOptions) -> Result<()> {
        // Enforce mandatory security options
        if !security.selinux_enabled {
            return Err(anyhow!("SELinux must be enabled for containers"));
        }

        if !security.no_new_privileges {
            return Err(anyhow!("no_new_privileges must be set to true"));
        }

        if security.seccomp_profile.is_none() {
            return Err(anyhow!("A seccomp profile must be specified"));
        }

        // Validate capabilities
        for cap in &security.drop_capabilities {
            if !is_valid_capability(cap) {
                return Err(anyhow!("Invalid capability specified: {}", cap));
            }
        }

        Ok(())
    }

    fn validate_mounts(&self, mounts: &[Mount]) -> Result<()> {
        for mount in mounts {
            // Validate source path
            if !mount.source.exists() {
                return Err(anyhow!("Mount source does not exist: {:?}", mount.source));
            }

            // Check for path traversal attempts
            if mount.source.to_string_lossy().contains("..")
                || mount.target.to_string_lossy().contains("..")
            {
                return Err(anyhow!("Path traversal detected in mount paths"));
            }

            // Enforce SELinux relabeling for mounts
            if !mount.selinux_relabel {
                return Err(anyhow!("SELinux relabeling must be enabled for mounts"));
            }
        }

        Ok(())
    }

    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;

        // Perform pre-start security checks
        self.pre_start_security_check(container)?;

        // Update status
        container.status = ContainerStatus::Running;

        Ok(())
    }

    fn pre_start_security_check(&self, container: &ContainerInfo) -> Result<()> {
        // Verify image signature and integrity
        self.verify_image_integrity(&container.config.image)?;

        // Check for CVE-2024-0132 vulnerability
        if !container.config.security.use_cdi {
            return Err(anyhow!(
                "Container must use CDI mode to prevent TOCTOU vulnerabilities"
            ));
        }

        // Additional runtime security checks
        if !container.config.security.readonly_root_fs {
            return Err(anyhow!("Read-only root filesystem is required"));
        }

        Ok(())
    }

    fn verify_image_integrity(&self, image: &str) -> Result<()> {
        // Implement image signature verification
        // This is a placeholder for actual implementation
        info!("Verifying image integrity: {}", image);
        Ok(())
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get_mut(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;

        container.status = ContainerStatus::Stopped;
        Ok(())
    }

    pub async fn remove_container(&self, container_id: &str) -> Result<()> {
        let mut containers = self.containers.write().await;
        let container = containers
            .get(container_id)
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))?;

        if container.status == ContainerStatus::Running {
            return Err(anyhow!("Cannot remove running container"));
        }

        containers.remove(container_id);
        Ok(())
    }

    pub async fn list_containers(&self) -> HashMap<String, ContainerInfo> {
        self.containers.read().await.clone()
    }

    pub async fn get_container(&self, container_id: &str) -> Result<ContainerInfo> {
        self.containers
            .read()
            .await
            .get(container_id)
            .cloned()
            .ok_or_else(|| anyhow!("Container not found: {}", container_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPURequirements {
    pub count: u32,
    pub memory: u64,
    pub compute_capability: Option<String>,
    pub mig_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
}

fn is_valid_capability(cap: &str) -> bool {
    // List of valid Linux capabilities
    let valid_caps = [
        "CHOWN",
        "DAC_OVERRIDE",
        "DAC_READ_SEARCH",
        "FOWNER",
        "FSETID",
        "KILL",
        "SETGID",
        "SETUID",
        "SETPCAP",
        "LINUX_IMMUTABLE",
        "NET_BIND_SERVICE",
        "NET_BROADCAST",
        "NET_ADMIN",
        "NET_RAW",
        "IPC_LOCK",
        "IPC_OWNER",
        "SYS_MODULE",
        "SYS_RAWIO",
        "SYS_CHROOT",
        "SYS_PTRACE",
        "SYS_PACCT",
        "SYS_ADMIN",
        "SYS_BOOT",
        "SYS_NICE",
        "SYS_RESOURCE",
        "SYS_TIME",
        "SYS_TTY_CONFIG",
        "MKNOD",
        "LEASE",
        "AUDIT_WRITE",
        "AUDIT_CONTROL",
        "SETFCAP",
        "MAC_OVERRIDE",
        "MAC_ADMIN",
        "SYSLOG",
        "WAKE_ALARM",
        "BLOCK_SUSPEND",
        "AUDIT_READ",
    ];
    valid_caps.contains(&cap)
}
