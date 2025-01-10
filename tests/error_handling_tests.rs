use actix_web::{test, ResponseError};
use document_automation::error::AppError;

#[actix_web::test]
async fn test_error_status_codes() {
    let test_cases = vec![
        (AppError::NotFound("test".to_string()), 404),
        (AppError::Internal("test".to_string()), 500),
        (AppError::TooManyRequests("test".to_string()), 429),
    ];

    for (error, expected_status) in test_cases {
        let response = error.error_response();
        assert_eq!(response.status().as_u16(), expected_status);
    }
} 