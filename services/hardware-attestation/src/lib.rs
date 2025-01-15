pub mod ffi;
pub use ffi::nvml_bindings::NvmlWrapper;

impl Clone for NvmlWrapper {
    fn clone(&self) -> Self {
        Self::new().expect("Failed to initialize NVML")
    }
}
