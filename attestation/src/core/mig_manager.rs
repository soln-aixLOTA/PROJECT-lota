use anyhow::{anyhow, Result};
use log::{error, info};
use nvml_wrapper::enum_wrappers::device::ComputeMode;
use nvml_wrapper::Device;
use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigProfile {
    pub gpu_instance_size: u32,
    pub compute_instance_size: u32,
    pub memory_size: u64,
    pub max_instances: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigInstance {
    pub id: String,
    pub gpu_id: String,
    pub profile: MigProfile,
    pub allocated: bool,
    pub tenant_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigConfiguration {
    pub gpu_id: String,
    pub enabled: bool,
    pub instances: Vec<MigInstance>,
    pub max_instances: u32,
}

pub struct MigManager {
    nvml: Arc<Nvml>,
    configurations: Arc<RwLock<HashMap<String, MigConfiguration>>>,
}

impl MigManager {
    pub fn new() -> Result<Self> {
        let nvml = Nvml::init()?;
        Ok(Self {
            nvml: Arc::new(nvml),
            configurations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        // Get device count
        let device_count = self.nvml.device_count()?;

        // Initialize configurations for each GPU
        let mut configs = self.configurations.write().await;
        for i in 0..device_count {
            let device = self.nvml.device_by_index(i)?;
            let uuid = device.uuid()?.to_string();

            // Check if MIG is enabled by checking compute mode
            let compute_mode = device.compute_mode()?;
            let enabled = compute_mode == ComputeMode::Default;

            // Get maximum number of MIG instances
            let max_instances = if enabled {
                // For A100-SXM4-40GB, we can have up to 7 1g.5gb instances
                7
            } else {
                0
            };

            configs.insert(
                uuid.clone(),
                MigConfiguration {
                    gpu_id: uuid,
                    enabled,
                    instances: Vec::new(),
                    max_instances,
                },
            );
        }

        Ok(())
    }

    pub async fn enable_mig(&self, gpu_id: &str) -> Result<()> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        if config.enabled {
            return Ok(());
        }

        // Get device handle
        let mut device = self.get_device(gpu_id)?;

        // Enable MIG mode by setting compute mode to Default
        device.set_compute_mode(ComputeMode::Default)?;

        // Update configuration
        config.enabled = true;
        config.max_instances = 7; // For A100-SXM4-40GB

        info!("MIG mode enabled for GPU {}", gpu_id);
        Ok(())
    }

    pub async fn disable_mig(&self, gpu_id: &str) -> Result<()> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        if !config.enabled {
            return Ok(());
        }

        // Check if there are any active instances
        if config.instances.iter().any(|i| i.allocated) {
            return Err(anyhow!("Cannot disable MIG while instances are in use"));
        }

        // Get device handle
        let mut device = self.get_device(gpu_id)?;

        // Disable MIG mode by setting compute mode to Exclusive Process
        device.set_compute_mode(ComputeMode::ExclusiveProcess)?;

        // Update configuration
        config.enabled = false;
        config.max_instances = 0;
        config.instances.clear();

        info!("MIG mode disabled for GPU {}", gpu_id);
        Ok(())
    }

    pub async fn create_instance(&self, gpu_id: &str, profile: MigProfile) -> Result<MigInstance> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        if !config.enabled {
            return Err(anyhow!("MIG mode not enabled for GPU {}", gpu_id));
        }

        if config.instances.len() >= config.max_instances as usize {
            return Err(anyhow!("Maximum number of MIG instances reached"));
        }

        // Create instance
        let instance = MigInstance {
            id: uuid::Uuid::new_v4().to_string(),
            gpu_id: gpu_id.to_string(),
            profile,
            allocated: false,
            tenant_id: None,
            created_at: chrono::Utc::now(),
        };

        config.instances.push(instance.clone());

        info!("Created MIG instance {} on GPU {}", instance.id, gpu_id);
        Ok(instance)
    }

    pub async fn destroy_instance(&self, gpu_id: &str, instance_id: &str) -> Result<()> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        let instance_idx = config
            .instances
            .iter()
            .position(|i| i.id == instance_id)
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        let instance = &config.instances[instance_idx];
        if instance.allocated {
            return Err(anyhow!("Cannot destroy allocated instance"));
        }

        // Remove instance from configuration
        config.instances.remove(instance_idx);

        info!("Destroyed MIG instance {} on GPU {}", instance_id, gpu_id);
        Ok(())
    }

    pub async fn allocate_instance(
        &self,
        gpu_id: &str,
        instance_id: &str,
        tenant_id: &str,
    ) -> Result<()> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        let instance = config
            .instances
            .iter_mut()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        if instance.allocated {
            return Err(anyhow!("Instance already allocated"));
        }

        instance.allocated = true;
        instance.tenant_id = Some(tenant_id.to_string());

        info!(
            "Allocated MIG instance {} on GPU {} to tenant {}",
            instance_id, gpu_id, tenant_id
        );
        Ok(())
    }

    pub async fn release_instance(&self, gpu_id: &str, instance_id: &str) -> Result<()> {
        let mut configs = self.configurations.write().await;
        let config = configs
            .get_mut(gpu_id)
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))?;

        let instance = config
            .instances
            .iter_mut()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| anyhow!("Instance not found: {}", instance_id))?;

        if !instance.allocated {
            return Ok(());
        }

        instance.allocated = false;
        instance.tenant_id = None;

        info!("Released MIG instance {} on GPU {}", instance_id, gpu_id);
        Ok(())
    }

    pub async fn get_configuration(&self, gpu_id: &str) -> Result<MigConfiguration> {
        let configs = self.configurations.read().await;
        configs
            .get(gpu_id)
            .cloned()
            .ok_or_else(|| anyhow!("GPU not found: {}", gpu_id))
    }

    pub async fn list_configurations(&self) -> HashMap<String, MigConfiguration> {
        self.configurations.read().await.clone()
    }

    fn get_device(&self, gpu_id: &str) -> Result<Device> {
        let device_count = self.nvml.device_count()?;
        for i in 0..device_count {
            let device = self.nvml.device_by_index(i)?;
            if device.uuid()?.to_string() == gpu_id {
                return Ok(device);
            }
        }
        Err(anyhow!("GPU not found: {}", gpu_id))
    }
}

impl Drop for MigManager {
    fn drop(&mut self) {
        // We can't call shutdown() on an Arc<Nvml>, so we'll just log any errors
        error!("MigManager dropped");
    }
}
