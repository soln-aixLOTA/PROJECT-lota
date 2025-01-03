use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Future, Ready};
use jsonschema::{Draft, JSONSchema};
use lazy_static::lazy_static;
use prometheus::{register_counter_vec, CounterVec};
use serde_json::Value;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tracing::{error, warn};

lazy_static! {
    static ref VALIDATION_FAILURES: CounterVec = register_counter_vec!(
        "api_validation_failures",
        "API validation failures by type",
        &["endpoint", "validation_type"]
    )
    .unwrap();

    static ref REQUEST_SCHEMAS: dashmap::DashMap<String, JSONSchema> = {
        let mut m = dashmap::DashMap::new();

        // Add your JSON schemas here
        let example_schema = serde_json::json!({
            "type": "object",
            "required": ["name", "description"],
            "properties": {
                "name": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 100
                },
                "description": {
                    "type": "string",
                    "maxLength": 1000
                }
            }
        });

        m.insert(
            "POST:/api/v1/example".to_string(),
            JSONSchema::options()
                .with_draft(Draft::Draft7)
                .compile(&example_schema)
                .expect("Invalid schema"),
        );

        m
    };
}

pub struct ValidationMiddleware;

impl ValidationMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for ValidationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ValidationMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ValidationMiddlewareService { service })
    }
}

pub struct ValidationMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ValidationMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();
        let endpoint_key = format!("{}:{}", method, path);

        // Validate request body if schema exists
        if let Some(schema) = REQUEST_SCHEMAS.get(&endpoint_key) {
            if let Some(body) = req.extensions_mut().remove::<Value>() {
                if let Err(errors) = schema.validate(&body) {
                    VALIDATION_FAILURES
                        .with_label_values(&[&endpoint_key, "schema"])
                        .inc();

                    let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
                    error!(
                        "Schema validation failed for {}: {:?}",
                        endpoint_key, error_messages
                    );

                    return Box::pin(async move {
                        Err(actix_web::error::ErrorBadRequest(format!(
                            "Validation error: {}",
                            error_messages.join(", ")
                        )))
                    });
                }

                // Put the body back
                req.extensions_mut().insert(body);
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    use actix_web::{web, App, HttpResponse};

    async fn test_handler(_: web::Json<Value>) -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[actix_web::test]
    async fn test_validation_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(ValidationMiddleware::new())
                .route("/api/v1/example", web::post().to(test_handler)),
        )
        .await;

        // Test valid request
        let valid_req = test::TestRequest::post()
            .uri("/api/v1/example")
            .set_json(serde_json::json!({
                "name": "Test",
                "description": "Test description"
            }))
            .to_request();

        let resp = test::call_service(&app, valid_req).await;
        assert!(resp.status().is_success());

        // Test invalid request
        let invalid_req = test::TestRequest::post()
            .uri("/api/v1/example")
            .set_json(serde_json::json!({
                "name": "",  // Invalid: empty string
                "description": "Test description"
            }))
            .to_request();

        let resp = test::call_service(&app, invalid_req).await;
        assert!(resp.status().is_client_error());
    }
}
