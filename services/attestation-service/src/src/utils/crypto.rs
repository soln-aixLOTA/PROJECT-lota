use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub fn verify_signature(message: &[u8], signature_b64: &str, public_key_b64: &str) -> bool {
    // Decode base64 signature
    let sig_bytes = match BASE64.decode(signature_b64) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Decode base64 public key
    let pk_bytes = match BASE64.decode(public_key_b64) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Convert bytes to ed25519 types
    let public_key = match VerifyingKey::from_bytes(&pk_bytes.try_into().unwrap_or([0u8; 32])) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let signature = match Signature::from_slice(&sig_bytes) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Verify the signature
    public_key.verify(message, &signature).is_ok()
}
