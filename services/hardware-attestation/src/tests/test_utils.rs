use crate::{
    error::{AttestationError, StorageError},
    AttestationResult, GpuDetail, HardwareAttestation, HardwareInfo, VerificationStatus,
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Mock state for simulating different scenarios
#[derive(Default)]
pub struct MockState {
    pub should_fail_verification: bool,
    pub should_fail_storage: bool,
    pub gpu_count: usize,
    pub stored_attestations: Vec<AttestationResult>,
}

// Mock implementation of HardwareAttestation
pub struct MockHardwareAttestation {
    state: Arc<Mutex<MockState>>,
}

impl MockHardwareAttestation {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockState {
                should_fail_verification: false,
                should_fail_storage: false,
                gpu_count: 2, // Default to 2 GPUs for testing
                stored_attestations: Vec::new(),
            })),
        }
    }

    pub fn with_error(mut self) -> Self {
        let state = Arc::get_mut(&mut self.state).unwrap();
        let mut state = futures::executor::block_on(state.lock());
        state.should_fail_verification = true;
        self
    }

    pub fn with_storage_error(mut self) -> Self {
        let state = Arc::get_mut(&mut self.state).unwrap();
        let mut state = futures::executor::block_on(state.lock());
        state.should_fail_storage = true;
        self
    }

    pub fn with_gpu_count(mut self, count: usize) -> Self {
        let state = Arc::get_mut(&mut self.state).unwrap();
        let mut state = futures::executor::block_on(state.lock());
        state.gpu_count = count;
        self
    }

    // Helper to generate mock GPU details
    fn generate_mock_gpu_details(count: usize) -> Vec<GpuDetail> {
        (0..count)
            .map(|i| GpuDetail {
                uuid: format!("GPU-{}-{}", i, Uuid::new_v4()),
                name: format!("NVIDIA Test GPU {}", i),
                memory_total: 8 * 1024 * 1024 * 1024, // 8GB
                compute_capability: "8.6".to_string(),
            })
            .collect()
    }
}

#[async_trait]
impl HardwareAttestation for MockHardwareAttestation {
    async fn verify_hardware(&self) -> Result<AttestationResult, AttestationError> {
        let state = self.state.lock().await;

        if state.should_fail_verification {
            return Err(AttestationError::HardwareVerificationError(
                "Mock verification failure".to_string(),
            ));
        }

        let gpu_details = Self::generate_mock_gpu_details(state.gpu_count);

        Ok(AttestationResult {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            hardware_info: HardwareInfo {
                gpu_count: state.gpu_count,
                gpu_details,
                driver_version: "535.129.03".to_string(),
            },
            verification_status: VerificationStatus::Verified,
        })
    }

    async fn store_attestation(&self, result: AttestationResult) -> Result<(), AttestationError> {
        let mut state = self.state.lock().await;

        if state.should_fail_storage {
            return Err(AttestationError::StorageError(StorageError::AwsError(
                "Mock storage failure".to_string(),
            )));
        }

        state.stored_attestations.push(result);
        Ok(())
    }
}

// Helper functions for creating test data
impl AttestationResult {
    pub fn mock() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            hardware_info: HardwareInfo {
                gpu_count: 2,
                gpu_details: MockHardwareAttestation::generate_mock_gpu_details(2),
                driver_version: "535.129.03".to_string(),
            },
            verification_status: VerificationStatus::Verified,
        }
    }

    pub fn mock_failed() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            hardware_info: HardwareInfo {
                gpu_count: 0,
                gpu_details: vec![],
                driver_version: "unknown".to_string(),
            },
            verification_status: VerificationStatus::Failed("Mock failure".to_string()),
        }
    }
}

// Test helpers for assertions
pub mod assertions {
    use super::*;

    pub async fn assert_attestation_stored(
        mock: &MockHardwareAttestation,
        expected_id: Uuid,
    ) -> bool {
        let state = mock.state.lock().await;
        state
            .stored_attestations
            .iter()
            .any(|attestation| attestation.id == expected_id)
    }

    pub async fn assert_gpu_count(mock: &MockHardwareAttestation, expected_count: usize) -> bool {
        let state = mock.state.lock().await;
        state
            .stored_attestations
            .iter()
            .any(|attestation| attestation.hardware_info.gpu_count == expected_count)
    }
}

// Re-export test utilities
pub use assertions::*;
