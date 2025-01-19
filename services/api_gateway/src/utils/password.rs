use crate::error::ApiError;
use zxcvbn::zxcvbn;

const MIN_PASSWORD_SCORE: u8 = 3;

pub fn validate_password(password: &str, user_inputs: &[&str]) -> Result<(), ApiError> {
    // Basic length check
    if password.len() < 8 {
        return Err(ApiError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    // Check password strength using zxcvbn
    let estimate = zxcvbn(password, user_inputs).map_err(|e| {
        ApiError::ValidationError(format!("Failed to evaluate password strength: {}", e))
    })?;

    if estimate.score() < MIN_PASSWORD_SCORE {
        let mut error_msg = String::from("Password is too weak. ");

        if let Some(feedback) = estimate.feedback() {
            if let Some(warning) = feedback.warning() {
                error_msg.push_str(&warning.to_string());
                error_msg.push_str(". ");
            }

            if !feedback.suggestions().is_empty() {
                error_msg.push_str("Suggestions: ");
                for suggestion in feedback.suggestions() {
                    error_msg.push_str(&suggestion.to_string());
                    error_msg.push_str(". ");
                }
            }
        }

        return Err(ApiError::ValidationError(error_msg));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_password() {
        let result = validate_password("password123", &["user@example.com"]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::ValidationError(_)));
    }

    #[test]
    fn test_short_password() {
        let result = validate_password("pass", &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::ValidationError(_)));
    }

    #[test]
    fn test_strong_password() {
        let result = validate_password("c0rr3ct!H0rs3!B@tt3ry!St@pl3", &[]);
        assert!(result.is_ok());
    }
}
