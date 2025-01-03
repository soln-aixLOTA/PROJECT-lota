use super::*;
use ed25519_dalek::{Keypair, Signer};
use rand::rngs::OsRng;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

async fn setup_test_db() -> Result<Arc<PgPool>, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/lota_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Clear test data
    sqlx::query!("DELETE FROM attestations")
        .execute(&pool)
        .await?;

    Ok(Arc::new(pool))
}

fn generate_test_keypair() -> (Keypair, String, String) {
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);
    let public_key_b64 = BASE64.encode(keypair.public.to_bytes());
    let signature_b64 = BASE64.encode(keypair.sign(b"test").to_bytes());
    (keypair, public_key_b64, signature_b64)
}

#[tokio::test]
async fn test_attestation_flow() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;
    let (keypair, public_key_b64, _) = generate_test_keypair();

    // Create test attestation data
    let attestation_data = serde_json::json!({
        "security_score": 85.5,
        "vulnerability_scan": {
            "status": "passed",
            "findings": []
        },
        "penetration_test": {
            "status": "passed",
            "findings": []
        }
    });

    // Sign the data
    let data_str = serde_json::to_string(&attestation_data)?;
    let signature = keypair.sign(data_str.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());

    // Create attestation request
    let request = AttestationRequest {
        model_id: Uuid::new_v4(),
        version_id: Uuid::new_v4(),
        attestation_type: "model_security".to_string(),
        attestation_data,
        signature: signature_b64,
        public_key: public_key_b64,
        metadata: None,
    };

    let attestation_service = AttestationService::new(pool.clone());
    let verification_service = VerificationService::new(pool.clone());

    // Create and verify attestation
    let attestation = attestation_service.create_attestation(request).await?;
    assert_eq!(attestation.status, "pending");

    let verified = verification_service
        .verify_attestation(attestation.id)
        .await?;
    assert!(verified);

    let final_attestation = attestation_service.get_attestation(attestation.id).await?;
    assert!(final_attestation.is_some());
    assert_eq!(final_attestation.unwrap().status, "verified");

    Ok(())
}

#[tokio::test]
async fn test_invalid_signature() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;
    let (_, public_key_b64, _) = generate_test_keypair();
    let (other_keypair, _, _) = generate_test_keypair();

    let attestation_data = serde_json::json!({
        "security_score": 85.5,
        "vulnerability_scan": { "status": "passed", "findings": [] },
        "penetration_test": { "status": "passed", "findings": [] }
    });

    // Sign with different keypair
    let data_str = serde_json::to_string(&attestation_data)?;
    let invalid_signature = other_keypair.sign(data_str.as_bytes());
    let invalid_signature_b64 = BASE64.encode(invalid_signature.to_bytes());

    let request = AttestationRequest {
        model_id: Uuid::new_v4(),
        version_id: Uuid::new_v4(),
        attestation_type: "model_security".to_string(),
        attestation_data,
        signature: invalid_signature_b64,
        public_key: public_key_b64,
        metadata: None,
    };

    let attestation_service = AttestationService::new(pool.clone());
    let result = attestation_service.create_attestation(request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), sqlx::Error::Protocol(_)));

    Ok(())
}

#[tokio::test]
async fn test_invalid_attestation_data() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;
    let (keypair, public_key_b64, _) = generate_test_keypair();

    // Missing required fields
    let invalid_data = serde_json::json!({
        "security_score": 85.5  // Missing vulnerability_scan and penetration_test
    });

    let data_str = serde_json::to_string(&invalid_data)?;
    let signature = keypair.sign(data_str.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());

    let request = AttestationRequest {
        model_id: Uuid::new_v4(),
        version_id: Uuid::new_v4(),
        attestation_type: "model_security".to_string(),
        attestation_data: invalid_data,
        signature: signature_b64,
        public_key: public_key_b64,
        metadata: None,
    };

    let attestation_service = AttestationService::new(pool.clone());
    let verification_service = VerificationService::new(pool.clone());

    // Should create but fail verification
    let attestation = attestation_service.create_attestation(request).await?;
    let verified = verification_service
        .verify_attestation(attestation.id)
        .await?;
    assert!(!verified);

    let final_attestation = attestation_service.get_attestation(attestation.id).await?;
    assert_eq!(final_attestation.unwrap().status, "failed");

    Ok(())
}

#[tokio::test]
async fn test_list_attestations() -> Result<(), Box<dyn std::error::Error>> {
    let pool = setup_test_db().await?;
    let (keypair, public_key_b64, _) = generate_test_keypair();
    let attestation_service = AttestationService::new(pool.clone());

    let model_id = Uuid::new_v4();
    let version_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];

    // Create multiple attestations
    for version_id in &version_ids {
        let data = serde_json::json!({
            "security_score": 85.5,
            "vulnerability_scan": { "status": "passed", "findings": [] },
            "penetration_test": { "status": "passed", "findings": [] }
        });

        let data_str = serde_json::to_string(&data)?;
        let signature = keypair.sign(data_str.as_bytes());
        let signature_b64 = BASE64.encode(signature.to_bytes());

        let request = AttestationRequest {
            model_id,
            version_id: *version_id,
            attestation_type: "model_security".to_string(),
            attestation_data: data,
            signature: signature_b64,
            public_key: public_key_b64.clone(),
            metadata: None,
        };

        attestation_service.create_attestation(request).await?;
    }

    // Test listing with filters
    let attestations = attestation_service
        .list_attestations(Some(model_id), None, Some("pending".to_string()), 10, 0)
        .await?;
    assert_eq!(attestations.len(), 3);

    let attestations = attestation_service
        .list_attestations(Some(model_id), Some(version_ids[0]), None, 10, 0)
        .await?;
    assert_eq!(attestations.len(), 1);
    assert_eq!(attestations[0].version_id, version_ids[0]);

    // Test pagination
    let attestations = attestation_service
        .list_attestations(Some(model_id), None, None, 2, 0)
        .await?;
    assert_eq!(attestations.len(), 2);

    let attestations = attestation_service
        .list_attestations(Some(model_id), None, None, 2, 2)
        .await?;
    assert_eq!(attestations.len(), 1);

    Ok(())
}
