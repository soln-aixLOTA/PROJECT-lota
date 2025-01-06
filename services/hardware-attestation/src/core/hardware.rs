use crate::error::AttestationError;
use nvml_wrapper::{enum_wrappers::device::TemperatureSensor, Device, Nvml};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub id: Uuid,
    pub gpu_count: usize,
    pub gpus: Vec<GpuInfo>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub memory_total: u64,
    pub compute_capability: String,
    pub power_usage: Option<u32>,
    pub temperature: Option<u32>,
}

pub struct HardwareVerifier {
    nvml: Arc<Mutex<Nvml>>,
}

impl HardwareVerifier {
    pub fn new() -> Result<Self, AttestationError> {
        let nvml = Nvml::init().map_err(|e| AttestationError::NvmlInitError(e.to_string()))?;

        Ok(Self {
            nvml: Arc::new(Mutex::new(nvml)),
        })
    }

    pub async fn verify_hardware(&self) -> Result<HardwareInfo, AttestationError> {
        let nvml = self.nvml.lock().await;
        let device_count = nvml
            .device_count()
            .map_err(|e| AttestationError::DeviceCountError(e.to_string()))?
            as usize;

        info!("Found {} NVIDIA GPU(s)", device_count);

        let mut gpus = Vec::with_capacity(device_count);

        for i in 0..device_count {
            let device =
                nvml.device_by_index(i as u32)
                    .map_err(|e| AttestationError::GpuAccessError {
                        index: i,
                        message: e.to_string(),
                    })?;

            let gpu_info =
                self.get_gpu_info(i, &device)
                    .map_err(|e| AttestationError::GpuAccessError {
                        index: i,
                        message: e.to_string(),
                    })?;

            debug!("GPU {}: {:?}", i, gpu_info);
            gpus.push(gpu_info);
        }

        Ok(HardwareInfo {
            id: Uuid::new_v4(),
            gpu_count: device_count,
            gpus,
            timestamp: chrono::Utc::now(),
        })
    }

    fn get_gpu_info(&self, index: usize, device: &Device) -> Result<GpuInfo, AttestationError> {
        let name = device.name().map_err(|e| {
            AttestationError::GpuInfoError(format!("Failed to get device name: {}", e))
        })?;

        let memory = device.memory_info().map_err(|e| {
            AttestationError::GpuInfoError(format!("Failed to get memory info: {}", e))
        })?;

        let compute_capability = device.cuda_compute_capability().map_err(|e| {
            AttestationError::GpuInfoError(format!("Failed to get compute capability: {}", e))
        })?;

        let power_usage = device
            .power_usage()
            .map_err(|e| {
                AttestationError::GpuInfoError(format!("Failed to get power usage: {}", e))
            })
            .ok();

        let temperature = device
            .temperature(TemperatureSensor::Gpu)
            .map_err(|e| {
                AttestationError::GpuInfoError(format!("Failed to get temperature: {}", e))
            })
            .ok();

        Ok(GpuInfo {
            index,
            name,
            memory_total: memory.total,
            compute_capability: format!(
                "{}.{}",
                compute_capability.major, compute_capability.minor
            ),
            power_usage,
            temperature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_hardware_verifier_creation() {
        let result = HardwareVerifier::new();
        assert!(
            result.is_ok(),
            "Should create HardwareVerifier successfully"
        );
    }

    // Note: More comprehensive tests would typically use mocking
    // but NVML makes this challenging. Consider integration tests
    // on actual hardware or with a custom test wrapper.
}
