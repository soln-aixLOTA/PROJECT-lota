use super::ffi::nvml_bindings::{NvmlWrapper, NvmlError};
use test_log::test;

#[test]
fn test_nvml_initialization() {
    let nvml = NvmlWrapper::new();
    assert!(nvml.is_ok(), "NVML initialization failed: {:?}", nvml.err());
}

#[test]
fn test_device_count() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    let count = nvml.get_device_count();
    assert!(count.is_ok(), "Failed to get device count: {:?}", count.err());
    assert!(count.unwrap() >= 0, "Invalid device count");
}

#[test]
fn test_memory_info() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    let count = nvml.get_device_count().expect("Failed to get device count");

    if count > 0 {
        let memory = nvml.get_device_memory_info(0);
        assert!(memory.is_ok(), "Failed to get memory info: {:?}", memory.err());

        let memory = memory.unwrap();
        assert!(memory.total > 0, "Invalid total memory");
        assert!(memory.used <= memory.total, "Used memory exceeds total");
        assert!(memory.free <= memory.total, "Free memory exceeds total");
        assert_eq!(memory.used + memory.free, memory.total, "Memory accounting mismatch");
    }
}

#[test]
fn test_invalid_device_index() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    let result = nvml.get_device_memory_info(999);
    assert!(matches!(result, Err(NvmlError::DeviceNotFound)));
}

#[test]
fn test_multiple_initializations() {
    // First initialization should succeed
    let nvml1 = NvmlWrapper::new();
    assert!(nvml1.is_ok(), "First initialization failed");

    // Second initialization should also succeed (NVML handles this internally)
    let nvml2 = NvmlWrapper::new();
    assert!(nvml2.is_ok(), "Second initialization failed");

    // Both instances should be able to query devices
    let count1 = nvml1.unwrap().get_device_count();
    let count2 = nvml2.unwrap().get_device_count();

    assert_eq!(count1.ok(), count2.ok(), "Device counts don't match");
}

#[test]
fn test_error_handling() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");

    // Test various error conditions
    let invalid_device = nvml.get_device_memory_info(u32::MAX);
    assert!(matches!(invalid_device, Err(NvmlError::DeviceNotFound)));

    // Test error conversion
    let unknown_error = NvmlError::from(-999);
    assert!(matches!(unknown_error, NvmlError::Unknown(-999)));
}

#[tokio::test]
async fn test_async_usage() {
    // Verify the wrapper can be used in async contexts
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");

    let handle = tokio::spawn(async move {
        let count = nvml.get_device_count()?;
        Ok::<i32, NvmlError>(count)
    });

    let result = handle.await.expect("Task panicked");
    assert!(result.is_ok(), "Async device count failed: {:?}", result.err());
}

#[test]
fn test_memory_consistency() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    let count = nvml.get_device_count().expect("Failed to get device count");

    if count > 0 {
        // Take multiple readings and ensure they're consistent
        let memory1 = nvml.get_device_memory_info(0).expect("First reading failed");
        let memory2 = nvml.get_device_memory_info(0).expect("Second reading failed");

        // Total memory should not change
        assert_eq!(memory1.total, memory2.total, "Total memory changed between readings");

        // Used + Free should always equal Total
        assert_eq!(memory1.used + memory1.free, memory1.total, "First reading memory mismatch");
        assert_eq!(memory2.used + memory2.free, memory2.total, "Second reading memory mismatch");
    }
}
