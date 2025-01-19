#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use lotabots_auth::PasswordResetRequest;
use validator::Validate;

#[derive(Debug, Arbitrary)]
struct FuzzPasswordResetInput {
    token: String,
    new_password: String,
}

impl From<FuzzPasswordResetInput> for PasswordResetRequest {
    fn from(input: FuzzPasswordResetInput) -> Self {
        PasswordResetRequest {
            token: input.token,
            new_password: input.new_password,
        }
    }
}

fuzz_target!(|input: FuzzPasswordResetInput| {
    // Convert fuzzer input to PasswordResetRequest
    let request: PasswordResetRequest = input.into();

    // Test validation
    let _ = request.validate();

    // Test serialization/deserialization
    if let Ok(json) = serde_json::to_string(&request) {
        let _ = serde_json::from_str::<PasswordResetRequest>(&json);
    }

    // Test token validation
    let token_valid = !request.token.is_empty()
        && request.token.len() <= 256
        && request.token.chars().all(|c| c.is_ascii());

    // Test password validation (similar to registration but more thorough)
    let password_valid = request.new_password.len() >= 8
        && request.new_password.len() <= 128
        && request.new_password.chars().any(|c| c.is_ascii_digit())
        && request.new_password.chars().any(|c| c.is_ascii_uppercase())
        && request.new_password.chars().any(|c| c.is_ascii_lowercase())
        && request.new_password.chars().any(|c| !c.is_alphanumeric())
        && !request.new_password.contains("password")
        && !request.new_password.contains("123456")
        && !request.new_password.contains("qwerty");

    // Test for common attack patterns
    let attack_patterns = [
        "<script>", // XSS
        "../../", // Path traversal
        "$()", // Command injection
        "${", // Template injection
        "'", "\"", ";", "--", // SQL injection
        "\u{0000}", // Null byte
        "\n", "\r", // Line breaks
    ];

    let contains_attack_pattern = attack_patterns.iter().any(|pattern| {
        request.token.contains(pattern) || request.new_password.contains(pattern)
    });

    // If input contains interesting patterns or valid fields, help libfuzzer focus
    if token_valid || password_valid || contains_attack_pattern {
        libfuzzer_sys::arbitrary::Arbitrary::default();
    }
});
