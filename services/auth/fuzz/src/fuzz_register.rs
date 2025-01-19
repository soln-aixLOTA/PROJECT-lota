#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use lotabots_auth::RegisterRequest;
use validator::Validate;

#[derive(Debug, Arbitrary)]
struct FuzzRegisterInput {
    username: String,
    email: String,
    password: String,
}

impl From<FuzzRegisterInput> for RegisterRequest {
    fn from(input: FuzzRegisterInput) -> Self {
        RegisterRequest {
            username: input.username,
            email: input.email,
            password: input.password,
        }
    }
}

fuzz_target!(|input: FuzzRegisterInput| {
    // Convert fuzzer input to RegisterRequest
    let request: RegisterRequest = input.into();

    // Test validation
    let validation_result = request.validate();

    // Test username validation
    let username_valid = request.username.len() >= 3
        && request.username.len() <= 50
        && request.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');

    // Test email validation
    let email_parts: Vec<&str> = request.email.split('@').collect();
    let email_valid = email_parts.len() == 2
        && !email_parts[0].is_empty()
        && email_parts[1].contains('.');

    // Test password validation
    let password_valid = request.password.len() >= 8
        && request.password.chars().any(|c| c.is_ascii_digit())
        && request.password.chars().any(|c| c.is_ascii_uppercase())
        && request.password.chars().any(|c| c.is_ascii_lowercase())
        && request.password.chars().any(|c| !c.is_alphanumeric());

    // If any validation passes, the input is interesting
    if username_valid || email_valid || password_valid {
        // This helps libfuzzer focus on inputs that pass some validations
        // but might still fail others
        if validation_result.is_err() {
            // Found an interesting case where some validations pass but others fail
            return;
        }
    }
});
