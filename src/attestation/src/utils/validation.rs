use serde_json::Value;
use validator::{Validate, ValidationError};

pub fn validate_attestation_data(
    data: &Value,
    attestation_type: &str,
) -> Result<(), ValidationError> {
    match attestation_type {
        "model_security" => validate_model_security(data),
        "model_performance" => validate_model_performance(data),
        "model_fairness" => validate_model_fairness(data),
        _ => Err(ValidationError::new("unsupported_attestation_type")),
    }
}

fn validate_model_security(data: &Value) -> Result<(), ValidationError> {
    // Check required fields
    let required_fields = ["vulnerability_scan", "penetration_test", "security_score"];
    for field in required_fields {
        if !data.get(field).is_some() {
            return Err(ValidationError::new("missing_required_field"));
        }
    }

    // Validate security score
    if let Some(score) = data.get("security_score").and_then(|v| v.as_f64()) {
        if score < 0.0 || score > 100.0 {
            return Err(ValidationError::new("invalid_security_score"));
        }
    }

    Ok(())
}

fn validate_model_performance(data: &Value) -> Result<(), ValidationError> {
    // Check required fields
    let required_fields = ["accuracy", "latency", "throughput"];
    for field in required_fields {
        if !data.get(field).is_some() {
            return Err(ValidationError::new("missing_required_field"));
        }
    }

    // Validate metrics
    if let Some(accuracy) = data.get("accuracy").and_then(|v| v.as_f64()) {
        if accuracy < 0.0 || accuracy > 1.0 {
            return Err(ValidationError::new("invalid_accuracy"));
        }
    }

    Ok(())
}

fn validate_model_fairness(data: &Value) -> Result<(), ValidationError> {
    // Check required fields
    let required_fields = ["bias_metrics", "fairness_score", "protected_attributes"];
    for field in required_fields {
        if !data.get(field).is_some() {
            return Err(ValidationError::new("missing_required_field"));
        }
    }

    // Validate fairness score
    if let Some(score) = data.get("fairness_score").and_then(|v| v.as_f64()) {
        if score < 0.0 || score > 1.0 {
            return Err(ValidationError::new("invalid_fairness_score"));
        }
    }

    Ok(())
}
