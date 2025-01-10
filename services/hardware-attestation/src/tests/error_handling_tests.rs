use super::test_utils::MockHardwareAttestation;
use crate::{
    error::{AttestationError, StorageError},
    AttestationResult, HardwareAttestation,
};
use std::error::Error;

#[tokio::test]
async fn test_hardware_verification_error_details() {
    let service = MockHardwareAttestation::new().with_error();
    let err = service.verify_hardware().await.unwrap_err();

    match err {
        AttestationError::HardwareVerificationError(msg) => {
            assert!(!msg.is_empty());
            assert!(msg.contains("Mock verification failure"));
        }
        _ => panic!("Expected HardwareVerificationError"),
    }
}

#[tokio::test]
async fn test_storage_error_details() {
    let service = MockHardwareAttestation::new().with_storage_error();
    let attestation = AttestationResult::mock();
    let err = service.store_attestation(attestation).await.unwrap_err();

    match err {
        AttestationError::StorageError(StorageError::AwsError(msg)) => {
            assert!(!msg.is_empty());
            assert!(msg.contains("Mock storage failure"));
        }
        _ => panic!("Expected StorageError::AwsError"),
    }
}

#[tokio::test]
async fn test_error_source_chain() {
    let service = MockHardwareAttestation::new().with_storage_error();
    let attestation = AttestationResult::mock();
    let err = service.store_attestation(attestation).await.unwrap_err();

    // Verify error chain
    let mut source_count = 0;
    let mut current_error = err.source();
    while let Some(source) = current_error {
        source_count += 1;
        current_error = source.source();
    }

    assert!(
        source_count > 0,
        "Error should have at least one source in chain"
    );
}

#[tokio::test]
async fn test_error_display_formatting() {
    let service = MockHardwareAttestation::new().with_error();
    let err = service.verify_hardware().await.unwrap_err();

    let error_string = format!("{}", err);
    assert!(!error_string.is_empty());
    assert!(error_string.contains("verification failed"));
}

#[tokio::test]
async fn test_error_debug_formatting() {
    let service = MockHardwareAttestation::new().with_error();
    let err = service.verify_hardware().await.unwrap_err();

    let debug_string = format!("{:?}", err);
    assert!(!debug_string.is_empty());
    assert!(debug_string.contains("HardwareVerificationError"));
}

#[tokio::test]
async fn test_multiple_error_scenarios() {
    let service = MockHardwareAttestation::new()
        .with_error()
        .with_storage_error();

    // Test both verification and storage errors
    let verification_err = service.verify_hardware().await.unwrap_err();
    assert!(matches!(
        verification_err,
        AttestationError::HardwareVerificationError(_)
    ));

    let storage_err = service
        .store_attestation(AttestationResult::mock())
        .await
        .unwrap_err();
    assert!(matches!(storage_err, AttestationError::StorageError(_)));
}

#[tokio::test]
async fn test_error_conversion() {
    // Test converting from StorageError to AttestationError
    let storage_error = StorageError::AwsError("test error".to_string());
    let attestation_error: AttestationError = storage_error.into();

    assert!(matches!(
        attestation_error,
        AttestationError::StorageError(_)
    ));
}

#[tokio::test]
async fn test_error_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    // Verify that our error types implement Send + Sync
    assert_send_sync::<AttestationError>();
    assert_send_sync::<StorageError>();
}
