#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_error_responses() {
        let test_cases = vec![
            (ApiError::Unauthorized, 401),
            (ApiError::NotFound, 404),
            (ApiError::Internal, 500),
            (ApiError::BadRequest("test".to_string()), 400),
            (ApiError::RateLimitExceeded, 429),
            (ApiError::ServiceUnavailable, 503),
        ];

        for (error, expected_status) in test_cases {
            let response = error.error_response();
            assert_eq!(response.status(), expected_status);
        }
    }
} 