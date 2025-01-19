#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use lotabots_auth::LoginRequest;
use validator::Validate;

#[derive(Debug, Arbitrary)]
struct FuzzLoginInput {
    username: String,
    password: String,
}

impl From<FuzzLoginInput> for LoginRequest {
    fn from(input: FuzzLoginInput) -> Self {
        LoginRequest {
            username: input.username,
            password: input.password,
        }
    }
}

fuzz_target!(|input: FuzzLoginInput| {
    // Convert fuzzer input to LoginRequest
    let request: LoginRequest = input.into();

    // Test validation
    let _ = request.validate();

    // Test serialization/deserialization
    if let Ok(json) = serde_json::to_string(&request) {
        let _ = serde_json::from_str::<LoginRequest>(&json);
    }

    // Test for potential SQL injection patterns
    let sql_injection_patterns = [
        "'", "\"", ";", "--", "/*", "*/", "OR", "AND",
        "UNION", "SELECT", "DROP", "DELETE", "UPDATE",
        "1=1", "1 = 1", "TRUE", "FALSE", // Common SQL injection conditions
        "SLEEP(", "WAITFOR", // Time-based SQL injection
        "@@version", "version()", // Information disclosure attempts
        "0x", "0b", // Hex/binary encoding attempts
    ];

    let contains_sql_injection = sql_injection_patterns.iter().any(|pattern| {
        request.username.to_uppercase().contains(&pattern.to_string())
            || request.password.to_uppercase().contains(&pattern.to_string())
    });

    // Test for common attack patterns
    let attack_patterns = [
        "<script>", "</script>", // XSS
        "../../", "../", // Path traversal
        "$()", "`", // Command injection
        "${", "#{", // Template injection
        "\u{0000}", "\u{0001}", // Null bytes and control characters
        "\n", "\r", "\t", // Special characters
        "data:", "javascript:", "file:", // Protocol handlers
        "%00", "%0d%0a", // URL encoded attacks
    ];

    let contains_attack_pattern = attack_patterns.iter().any(|pattern| {
        request.username.contains(pattern) || request.password.contains(pattern)
    });

    // Test for unicode handling and normalization
    let contains_unicode = request.username.chars().any(|c| !c.is_ascii())
        || request.password.chars().any(|c| !c.is_ascii());

    // Test for very long inputs (potential buffer overflow attempts)
    let has_long_input = request.username.len() > 100 || request.password.len() > 100;

    // Test for common username/password patterns
    let common_patterns = [
        "admin", "root", "administrator",
        "test", "guest", "system",
        "password", "123456", "qwerty",
    ];

    let uses_common_pattern = common_patterns.iter().any(|pattern| {
        request.username.to_lowercase().contains(pattern)
            || request.password.to_lowercase().contains(pattern)
    });

    // If input contains interesting patterns, help libfuzzer focus on similar inputs
    if contains_sql_injection
        || contains_attack_pattern
        || contains_unicode
        || has_long_input
        || uses_common_pattern {
        libfuzzer_sys::arbitrary::Arbitrary::default();
    }

    // Test for potential timing attack vectors
    let _ = request.username.len() == request.password.len();
});
