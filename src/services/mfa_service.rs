use crate::error::{AppError, AppResult};
use base32;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MfaService {
    period: u64,
    digits: usize,
}

impl Default for MfaService {
    fn default() -> Self {
        Self {
            period: 30,
            digits: 6,
        }
    }
}

impl MfaService {
    pub fn new(period: u64, digits: usize) -> Self {
        Self { period, digits }
    }

    pub fn generate_secret() -> AppResult<String> {
        let mut rng = rand::thread_rng();
        let mut secret = [0u8; 32];
        rng.fill_bytes(&mut secret);
        
        Ok(base32::encode(
            base32::Alphabet::RFC4648 { padding: true },
            &secret,
        ))
    }

    pub fn verify_totp(&self, secret: &str, code: &str) -> AppResult<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| AppError::Internal(format!("Failed to get system time: {}", e)))?
            .as_secs();

        // Check current and previous time step to account for clock skew
        for timestamp in [now, now - self.period] {
            let generated_code = self.generate_totp(secret, timestamp)?;
            if generated_code == code {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn generate_totp(&self, secret: &str, timestamp: u64) -> AppResult<String> {
        let decoded_secret = base32::decode(
            base32::Alphabet::RFC4648 { padding: true },
            secret,
        )
        .ok_or_else(|| AppError::Internal("Failed to decode secret".into()))?;

        let counter = timestamp / self.period;
        let counter_bytes = counter.to_be_bytes();

        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(&decoded_secret)
            .map_err(|e| AppError::Internal(format!("Failed to create HMAC: {}", e)))?;
        mac.update(&counter_bytes);
        let result = mac.finalize().into_bytes();

        let offset = (result[19] & 0xf) as usize;
        let code = ((result[offset] & 0x7f) as u32) << 24
            | (result[offset + 1] as u32) << 16
            | (result[offset + 2] as u32) << 8
            | result[offset + 3] as u32;

        let code = code % 10u32.pow(self.digits as u32);
        Ok(format!("{:0width$}", code, width = self.digits))
    }

    pub fn generate_provisioning_uri(&self, secret: &str, account_name: &str, issuer: &str) -> String {
        let encoded_account = urlencoding::encode(account_name);
        let encoded_issuer = urlencoding::encode(issuer);
        
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&period={}&digits={}",
            encoded_issuer, encoded_account, secret, encoded_issuer, self.period, self.digits
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_secret_generation() {
        let secret = MfaService::generate_secret().unwrap();
        assert!(!secret.is_empty());
        assert!(base32::decode(
            base32::Alphabet::RFC4648 { padding: true },
            &secret
        ).is_some());
    }

    #[test]
    fn test_totp_verification() {
        let service = MfaService::default();
        let secret = MfaService::generate_secret().unwrap();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let code = service.generate_totp(&secret, now).unwrap();
        assert!(service.verify_totp(&secret, &code).unwrap());
    }

    #[test]
    fn test_totp_expiration() {
        let service = MfaService::default();
        let secret = MfaService::generate_secret().unwrap();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let code = service.generate_totp(&secret, now).unwrap();
        
        // Wait for code to expire
        sleep(Duration::from_secs(31));
        
        assert!(!service.verify_totp(&secret, &code).unwrap());
    }

    #[test]
    fn test_provisioning_uri() {
        let service = MfaService::default();
        let secret = MfaService::generate_secret().unwrap();
        let uri = service.generate_provisioning_uri(&secret, "test@example.com", "TestApp");
        
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains(&secret));
        assert!(uri.contains("test%40example.com"));
        assert!(uri.contains("TestApp"));
    }
} 