use hardware_attestation::NvmlWrapper;
use std::hint::black_box;

fn memory_info_alloc() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    black_box(nvml.get_device_memory_info(0).expect("Failed to get memory info"));
}

fn mixed_operations_alloc() {
    let nvml = NvmlWrapper::new().expect("Failed to initialize NVML");
    black_box(nvml.get_device_memory_info(0).expect("Failed to get memory info"));
}

iai::main!(memory_info_alloc, mixed_operations_alloc);
