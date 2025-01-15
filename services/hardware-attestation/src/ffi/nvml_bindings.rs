#[derive(Debug)]
pub struct NvmlWrapper {
    _private: (),
}

#[derive(Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

impl NvmlWrapper {
    pub fn new() -> Result<Self, String> {
        Ok(Self { _private: () })
    }

    pub fn get_device_count(&self) -> Result<i32, String> {
        Ok(1)
    }

    pub fn get_device_memory_info(&self, _device_index: i32) -> Result<MemoryInfo, String> {
        Ok(MemoryInfo {
            total: 8 * 1024 * 1024 * 1024, // 8GB
            free: 4 * 1024 * 1024 * 1024,  // 4GB
            used: 4 * 1024 * 1024 * 1024,  // 4GB
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvml_wrapper() {
        let nvml = NvmlWrapper::new().unwrap();
        assert_eq!(nvml.get_device_count().unwrap(), 1);

        let memory = nvml.get_device_memory_info(0).unwrap();
        assert!(memory.total > 0);
        assert!(memory.used <= memory.total);
        assert_eq!(memory.total, memory.free + memory.used);
    }
}
