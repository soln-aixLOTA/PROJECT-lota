use anyhow::Result;
use log::{error, warn};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::Nvml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUMetrics {
    pub utilization: u32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub power_usage: u32,
    pub temperature: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUProcessInfo {
    pub pid: u32,
    pub memory_used: u64,
    pub gpu_utilization: u32,
    pub compute_instance_id: Option<u32>,
}

pub struct GPUMonitor {
    nvml: Arc<Nvml>,
    metrics: Arc<RwLock<HashMap<String, GPUMetrics>>>,
    update_interval: Duration,
    last_update: Arc<RwLock<Instant>>,
}

impl GPUMonitor {
    pub fn new(update_interval: Duration) -> Result<Self> {
        let nvml = Nvml::init()?;
        Ok(Self {
            nvml: Arc::new(nvml),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            update_interval,
            last_update: Arc::new(RwLock::new(Instant::now())),
        })
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        let nvml = Arc::clone(&self.nvml);
        let metrics = Arc::clone(&self.metrics);
        let update_interval = self.update_interval;
        let last_update = Arc::clone(&self.last_update);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(update_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::update_metrics(&nvml, &metrics, &last_update).await {
                    error!("Failed to update GPU metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn update_metrics(
        nvml: &Nvml,
        metrics: &RwLock<HashMap<String, GPUMetrics>>,
        last_update: &RwLock<Instant>,
    ) -> Result<()> {
        let mut metrics_map = metrics.write().await;
        *last_update.write().await = Instant::now();

        let device_count = nvml.device_count()?;
        for i in 0..device_count {
            let device = nvml.device_by_index(i)?;
            let uuid = device.uuid()?.to_string();

            let utilization = device.utilization_rates()?.gpu;
            let memory = device.memory_info()?;
            let power = device.power_usage()? / 1000; // Convert from milliwatts to watts
            let temp = device.temperature(TemperatureSensor::Gpu)?;

            let gpu_metrics = GPUMetrics {
                utilization,
                memory_used: memory.used,
                memory_total: memory.total,
                power_usage: power,
                temperature: temp,
                timestamp: chrono::Utc::now(),
            };

            // Check for concerning metrics
            if temp > 80 {
                warn!("High temperature ({} C) on GPU {}", temp, uuid);
            }
            if utilization > 95 {
                warn!("High utilization ({}%) on GPU {}", utilization, uuid);
            }
            if memory.used as f64 / memory.total as f64 > 0.95 {
                warn!(
                    "High memory usage ({}/{} MB) on GPU {}",
                    memory.used / 1024 / 1024,
                    memory.total / 1024 / 1024,
                    uuid
                );
            }

            metrics_map.insert(uuid, gpu_metrics);
        }

        Ok(())
    }

    pub async fn get_gpu_metrics(&self) -> HashMap<String, GPUMetrics> {
        self.metrics.read().await.clone()
    }

    pub async fn get_gpu_processes(&self) -> Result<HashMap<String, Vec<GPUProcessInfo>>> {
        let mut process_map = HashMap::new();
        let device_count = self.nvml.device_count()?;

        for i in 0..device_count {
            let device = self.nvml.device_by_index(i)?;
            let uuid = device.uuid()?.to_string();
            let mut processes = Vec::new();

            // Get compute processes
            if let Ok(compute_procs) = device.running_compute_processes() {
                for proc in compute_procs {
                    processes.push(GPUProcessInfo {
                        pid: proc.pid,
                        memory_used: match proc.used_gpu_memory {
                            UsedGpuMemory::Used(bytes) => bytes,
                            _ => 0,
                        },
                        gpu_utilization: 0, // Not available through NVML
                        compute_instance_id: proc.compute_instance_id,
                    });
                }
            }

            // Get graphics processes
            if let Ok(graphics_procs) = device.running_graphics_processes() {
                for proc in graphics_procs {
                    processes.push(GPUProcessInfo {
                        pid: proc.pid,
                        memory_used: match proc.used_gpu_memory {
                            UsedGpuMemory::Used(bytes) => bytes,
                            _ => 0,
                        },
                        gpu_utilization: 0, // Not available through NVML
                        compute_instance_id: proc.compute_instance_id,
                    });
                }
            }

            process_map.insert(uuid, processes);
        }

        Ok(process_map)
    }

    pub async fn check_gpu_health(&self) -> Result<HashMap<String, Vec<HealthCheck>>> {
        let mut health_map = HashMap::new();
        let device_count = self.nvml.device_count()?;

        for i in 0..device_count {
            let device = self.nvml.device_by_index(i)?;
            let uuid = device.uuid()?.to_string();
            let mut checks = Vec::new();

            // Check temperature
            let temp = device.temperature(TemperatureSensor::Gpu)?;
            if temp > 80 {
                checks.push(HealthCheck {
                    check_type: HealthCheckType::Temperature,
                    status: HealthStatus::Warning,
                    message: format!("High temperature: {} C", temp),
                });
            }

            // Check memory usage
            let memory = device.memory_info()?;
            let memory_usage = memory.used as f64 / memory.total as f64;
            if memory_usage > 0.95 {
                checks.push(HealthCheck {
                    check_type: HealthCheckType::Memory,
                    status: HealthStatus::Warning,
                    message: format!(
                        "High memory usage: {:.1}% ({}/{} MB)",
                        memory_usage * 100.0,
                        memory.used / 1024 / 1024,
                        memory.total / 1024 / 1024
                    ),
                });
            }

            // Check utilization
            let utilization = device.utilization_rates()?.gpu;
            if utilization > 95 {
                checks.push(HealthCheck {
                    check_type: HealthCheckType::Utilization,
                    status: HealthStatus::Warning,
                    message: format!("High GPU utilization: {}%", utilization),
                });
            }

            // Check power usage
            let power = device.power_usage()? / 1000;
            let power_limit = device.enforced_power_limit()? / 1000;
            if power as f64 / power_limit as f64 > 0.95 {
                checks.push(HealthCheck {
                    check_type: HealthCheckType::Power,
                    status: HealthStatus::Warning,
                    message: format!("High power usage: {} W (limit: {} W)", power, power_limit),
                });
            }

            if checks.is_empty() {
                checks.push(HealthCheck {
                    check_type: HealthCheckType::Overall,
                    status: HealthStatus::OK,
                    message: "All metrics within normal ranges".to_string(),
                });
            }

            health_map.insert(uuid, checks);
        }

        Ok(health_map)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    Temperature,
    Memory,
    Utilization,
    Power,
    Overall,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    OK,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub check_type: HealthCheckType,
    pub status: HealthStatus,
    pub message: String,
}
