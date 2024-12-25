use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage, http::{Method, StatusCode}};
use futures::future::{ok, Ready};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use jsonschema::{JSONSchema, Draft};
use lazy_static::lazy_static;
use std::collections::HashMap;
use prometheus::{IntCounter, register_int_counter};
use tracing::{warn, debug};

use crate::error::{ApiError, handle_validation_error};

lazy_static! {
    static ref VALIDATION_FAILURES: IntCounter = register_int_counter!(
        "api_gateway_validation_failures_total",
        "Total number of request validation failures",
        &["endpoint", "validation_type"]
    ).unwrap();

    static ref REQUEST_SCHEMAS: HashMap<String, JSONSchema> = {
        let mut m = HashMap::new();
        
        // Inference request schema
        m.insert(
            "POST:/api/v1/inference".to_string(),
            JSONSchema::compile(&serde_json::json!({
                "type": "object",
                "required": ["model_id", "inputs"],
                "properties": {
                    "model_id": { "type": "string" },
                    "inputs": { "type": "array" },
                    "parameters": { "type": "object" }
                }
            })).expect("Invalid inference schema")
        );

        // Chat request schema
        m.insert(
            "POST:/api/v1/chat".to_string(),
            JSONSchema::compile(&serde_json::json!({
                "type": "object",
                "required": ["messages"],
                "properties": {
                    "messages": {
                        "type": "array",
                        "minItems": 1,
                        "items": {
                            "type": "object",
                            "required": ["role", "content"],
                            "properties": {
                                "role": {
                                    "type": "string",
                                    "enum": ["user", "assistant", "system"]
                                },
                                "content": { "type": "string" }
                            }
                        }
                    },
                    "model": { "type": "string" },
                    "temperature": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 2
                    }
                }
            })).expect("Invalid chat schema")
        );

        // User creation schema
        m.insert(
            "POST:/api/v1/users".to_string(),
            JSONSchema::compile(&serde_json::json!({
                "type": "object",
                "required": ["email", "password"],
                "properties": {
                    "email": {
                        "type": "string",
                        "format": "email"
                    },
                    "password": {
                        "type": "string",
                        "minLength": 8
                    },
                    "name": { "type": "string" },
                    "organization": { "type": "string" }
                }
            })).expect("Invalid user schema")
        );

        m
    };

    static ref REQUIRED_HEADERS: HashMap<String, Vec<&'static str>> = {
        let mut m = HashMap::new();
        
        // All endpoints require these headers
        let base_headers = vec!["X-Request-ID"];
        
        // Authenticated endpoints require additional headers
        let auth_headers = vec!["Authorization", "X-Request-ID"];
        
        m.insert("GET:/api/v1/models".to_string(), base_headers.clone());
        m.insert("POST:/api/v1/inference".to_string(), auth_headers.clone());
        m.insert("POST:/api/v1/chat".to_string(), auth_headers.clone());
        m.insert("POST:/api/v1/users".to_string(), base_headers.clone());
        
        m
    };
}

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    max_body_size: usize,
    max_uri_length: usize,
    allowed_content_types: Vec<String>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_body_size: 1024 * 1024,  // 1MB
            max_uri_length: 2048,        // 2KB
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ],
        }
    }
}

pub struct ValidationMiddleware {
    config: ValidationConfig,
}

impl ValidationMiddleware {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }
}

impl<S> Transform<S, ServiceRequest> for ValidationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = ValidationMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ValidationMiddlewareService {
            service,
            config: self.config.clone(),
        })
    }
}

pub struct ValidationMiddlewareService<S> {
    service: S,
    config: ValidationConfig,
}

impl<S> Service<ServiceRequest> for ValidationMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let endpoint_key = format!("{}:{}", req.method(), req.path());
        let config = self.config.clone();

        // Validate URI length
        if req.uri().to_string().len() > config.max_uri_length {
            VALIDATION_FAILURES.with_label_values(&[
                &endpoint_key,
                "uri_length"
            ]).inc();

            return Box::pin(async move {
                Err(handle_validation_error(
                    "URI too long",
                    Some(json!({ "max_length": config.max_uri_length }))
                ).into())
            });
        }

        // Validate required headers
        if let Some(required_headers) = REQUIRED_HEADERS.get(&endpoint_key) {
            for header in required_headers {
                if !req.headers().contains_key(*header) {
                    VALIDATION_FAILURES.with_label_values(&[
                        &endpoint_key,
                        "missing_header"
                    ]).inc();

                    return Box::pin(async move {
                        Err(handle_validation_error(
                            format!("Missing required header: {}", header),
                            Some(json!({ "header": header }))
                        ).into())
                    });
                }
            }
        }

        // Validate content type for POST/PUT/PATCH requests
        if matches!(req.method(), Method::POST | Method::PUT | Method::PATCH) {
            if let Some(content_type) = req.headers().get("content-type") {
                let content_type = content_type.to_str().unwrap_or("");
                if !config.allowed_content_types.iter().any(|t| content_type.starts_with(t)) {
                    VALIDATION_FAILURES.with_label_values(&[
                        &endpoint_key,
                        "content_type"
                    ]).inc();

                    return Box::pin(async move {
                        Err(handle_validation_error(
                            "Unsupported content type",
                            Some(json!({
                                "content_type": content_type,
                                "allowed_types": config.allowed_content_types
                            }))
                        ).into())
                    });
                }
            }
        }

        // Schema validation for known endpoints
        if let Some(schema) = REQUEST_SCHEMAS.get(&endpoint_key) {
            let fut = self.service.call(req);
            
            Box::pin(async move {
                let mut response = fut.await?;
                
                if let Some(body) = response.response().body().as_ref() {
                    if let Ok(json) = serde_json::from_slice::<Value>(body) {
                        if let Err(errors) = schema.validate(&json) {
                            VALIDATION_FAILURES.with_label_values(&[
                                &endpoint_key,
                                "schema"
                            ]).inc();

                            return Err(handle_validation_error(
                                "Request body validation failed",
                                Some(json!({
                                    "errors": errors.collect::<Vec<_>>()
                                }))
                            ).into());
                        }
                    }
                }
                
                Ok(response)
            })
        } else {
            // No schema validation required
            Box::pin(self.service.call(req))
        }
    }
}