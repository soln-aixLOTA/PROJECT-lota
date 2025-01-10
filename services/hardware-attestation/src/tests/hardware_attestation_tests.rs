use super::test_utils::{assert_attestation_stored, assert_gpu_count, MockHardwareAttestation};
use crate::{error::AttestationError, AttestationResult, HardwareAttestation, VerificationStatus};

#[tokio::test]
async fn test_successful_hardware_verification() {
    let service = MockHardwareAttestation::new();
    let result = service.verify_hardware().await.unwrap();

    assert!(matches!(
        result.verification_status,
        VerificationStatus::Verified
    ));
    assert_eq!(result.hardware_info.gpu_count, 2); // Default mock has 2 GPUs
    assert_eq!(result.hardware_info.gpu_details.len(), 2);
    assert!(!result.hardware_info.driver_version.is_empty());
}

#[tokio::test]
async fn test_hardware_verification_with_different_gpu_count() {
    let service = MockHardwareAttestation::new().with_gpu_count(4);
    let result = service.verify_hardware().await.unwrap();

    assert!(matches!(
        result.verification_status,
        VerificationStatus::Verified
    ));
    assert_eq!(result.hardware_info.gpu_count, 4);
    assert_eq!(result.hardware_info.gpu_details.len(), 4);
}

#[tokio::test]
async fn test_hardware_verification_failure() {
    let service = MockHardwareAttestation::new().with_error();
    let result = service.verify_hardware().await;

    assert!(matches!(
        result.unwrap_err(),
        AttestationError::HardwareVerificationError(_)
    ));
}

#[tokio::test]
async fn test_successful_attestation_storage() {
    let service = MockHardwareAttestation::new();
    let attestation = AttestationResult::mock();
    let attestation_id = attestation.id;

    service.store_attestation(attestation).await.unwrap();
    assert!(assert_attestation_stored(&service, attestation_id).await);
}

#[tokio::test]
async fn test_attestation_storage_failure() {
    let service = MockHardwareAttestation::new().with_storage_error();
    let attestation = AttestationResult::mock();

    let result = service.store_attestation(attestation).await;
    assert!(matches!(
        result.unwrap_err(),
        AttestationError::StorageError(_)
    ));
}

#[tokio::test]
async fn test_full_attestation_workflow() {
    let service = MockHardwareAttestation::new();

    // First verify hardware
    let verification_result = service.verify_hardware().await.unwrap();
    assert!(matches!(
        verification_result.verification_status,
        VerificationStatus::Verified
    ));

    // Then store the attestation
    service
        .store_attestation(verification_result.clone())
        .await
        .unwrap();

    // Verify it was stored
    assert!(assert_attestation_stored(&service, verification_result.id).await);
    assert!(assert_gpu_count(&service, verification_result.hardware_info.gpu_count).await);
}

#[tokio::test]
async fn test_gpu_details_validation() {
    let service = MockHardwareAttestation::new();
    let result = service.verify_hardware().await.unwrap();

    // Validate GPU details
    for gpu in result.hardware_info.gpu_details {
        assert!(!gpu.uuid.is_empty());
        assert!(gpu.uuid.starts_with("GPU-"));
        assert!(!gpu.name.is_empty());
        assert!(gpu.name.contains("NVIDIA"));
        assert!(gpu.memory_total > 0);
        assert!(!gpu.compute_capability.is_empty());
    }
}

#[tokio::test]
async fn test_zero_gpu_scenario() {
    let service = MockHardwareAttestation::new().with_gpu_count(0);
    let result = service.verify_hardware().await.unwrap();

    assert_eq!(result.hardware_info.gpu_count, 0);
    assert!(result.hardware_info.gpu_details.is_empty());
    assert!(!result.hardware_info.driver_version.is_empty());
}
