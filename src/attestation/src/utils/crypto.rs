use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ed25519_dalek::{PublicKey, Signature, Verifier};

pub fn verify_signature(data: &str, signature: &str, public_key: &str) -> bool {
    // Decode base64 signature and public key
    let sig_bytes = match BASE64.decode(signature) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    let key_bytes = match BASE64.decode(public_key) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };

    // Parse public key and signature
    let public_key = match PublicKey::from_bytes(&key_bytes) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let signature = match Signature::from_bytes(&sig_bytes) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Verify signature
    public_key.verify(data.as_bytes(), &signature).is_ok()
}
