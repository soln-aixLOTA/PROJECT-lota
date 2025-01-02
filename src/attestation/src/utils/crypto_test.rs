use super::*;
use ed25519_dalek::{Keypair, Signer};
use rand::rngs::OsRng;

#[test]
fn test_valid_signature_verification() {
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);

    let data = "test data";
    let signature = keypair.sign(data.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());
    let public_key_b64 = BASE64.encode(keypair.public.to_bytes());

    assert!(verify_signature(data, &signature_b64, &public_key_b64));
}

#[test]
fn test_invalid_signature() {
    let mut csprng = OsRng;
    let keypair1: Keypair = Keypair::generate(&mut csprng);
    let keypair2: Keypair = Keypair::generate(&mut csprng);

    let data = "test data";
    let signature = keypair1.sign(data.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());
    let public_key_b64 = BASE64.encode(keypair2.public.to_bytes()); // Different key

    assert!(!verify_signature(data, &signature_b64, &public_key_b64));
}

#[test]
fn test_tampered_data() {
    let mut csprng = OsRng;
    let keypair: Keypair = Keypair::generate(&mut csprng);

    let original_data = "test data";
    let signature = keypair.sign(original_data.as_bytes());
    let signature_b64 = BASE64.encode(signature.to_bytes());
    let public_key_b64 = BASE64.encode(keypair.public.to_bytes());

    let tampered_data = "tampered data";
    assert!(!verify_signature(
        tampered_data,
        &signature_b64,
        &public_key_b64
    ));
}

#[test]
fn test_invalid_base64() {
    let data = "test data";

    // Invalid signature base64
    assert!(!verify_signature(data, "invalid base64!", "validbase64"));

    // Invalid public key base64
    assert!(!verify_signature(data, "validbase64", "invalid base64!"));
}

#[test]
fn test_invalid_key_format() {
    let data = "test data";
    let invalid_bytes = BASE64.encode(&[0u8; 32]); // Wrong key format
    let valid_signature = BASE64.encode(&[1u8; 64]); // Any valid length signature

    assert!(!verify_signature(data, &valid_signature, &invalid_bytes));
}
