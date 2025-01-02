use super::*;

#[test]
fn test_model_security_validation() {
    // Valid data
    let valid_data = serde_json::json!({
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
    assert!(validate_attestation_data(&valid_data, "model_security").is_ok());

    // Missing required field
    let missing_field = serde_json::json!({
        "security_score": 85.5,
        "vulnerability_scan": {
            "status": "passed",
            "findings": []
        }
        // missing penetration_test
    });
    assert!(validate_attestation_data(&missing_field, "model_security").is_err());

    // Invalid score range
    let invalid_score = serde_json::json!({
        "security_score": 150.0, // > 100
        "vulnerability_scan": {
            "status": "passed",
            "findings": []
        },
        "penetration_test": {
            "status": "passed",
            "findings": []
        }
    });
    assert!(validate_attestation_data(&invalid_score, "model_security").is_err());
}

#[test]
fn test_model_performance_validation() {
    // Valid data
    let valid_data = serde_json::json!({
        "accuracy": 0.95,
        "latency": {
            "p50": 10,
            "p95": 20,
            "p99": 30
        },
        "throughput": 1000
    });
    assert!(validate_attestation_data(&valid_data, "model_performance").is_ok());

    // Missing required field
    let missing_field = serde_json::json!({
        "accuracy": 0.95,
        "latency": {
            "p50": 10,
            "p95": 20,
            "p99": 30
        }
        // missing throughput
    });
    assert!(validate_attestation_data(&missing_field, "model_performance").is_err());

    // Invalid accuracy range
    let invalid_accuracy = serde_json::json!({
        "accuracy": 1.5, // > 1.0
        "latency": {
            "p50": 10,
            "p95": 20,
            "p99": 30
        },
        "throughput": 1000
    });
    assert!(validate_attestation_data(&invalid_accuracy, "model_performance").is_err());
}

#[test]
fn test_model_fairness_validation() {
    // Valid data
    let valid_data = serde_json::json!({
        "bias_metrics": {
            "demographic_parity": 0.95,
            "equal_opportunity": 0.92
        },
        "fairness_score": 0.93,
        "protected_attributes": ["gender", "age", "ethnicity"]
    });
    assert!(validate_attestation_data(&valid_data, "model_fairness").is_ok());

    // Missing required field
    let missing_field = serde_json::json!({
        "bias_metrics": {
            "demographic_parity": 0.95,
            "equal_opportunity": 0.92
        },
        "fairness_score": 0.93
        // missing protected_attributes
    });
    assert!(validate_attestation_data(&missing_field, "model_fairness").is_err());

    // Invalid fairness score range
    let invalid_score = serde_json::json!({
        "bias_metrics": {
            "demographic_parity": 0.95,
            "equal_opportunity": 0.92
        },
        "fairness_score": 1.5, // > 1.0
        "protected_attributes": ["gender", "age", "ethnicity"]
    });
    assert!(validate_attestation_data(&invalid_score, "model_fairness").is_err());
}

#[test]
fn test_unsupported_attestation_type() {
    let data = serde_json::json!({
        "some_field": "some_value"
    });
    let result = validate_attestation_data(&data, "unsupported_type");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "unsupported_attestation_type");
}
