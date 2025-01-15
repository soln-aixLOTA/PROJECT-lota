use actix_web::{
    route,
    web,
    HttpRequest,
    HttpResponse,
    Error,
};
use reqwest::Client;
use std::collections::HashMap;

pub struct ServiceRegistry {
    services: HashMap<String, String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let mut services = HashMap::new();
        services.insert("auth".to_string(), "http://auth:8080".to_string());
        services.insert("attestation".to_string(), "http://attestation:8080".to_string());
        services.insert("document".to_string(), "http://document:8080".to_string());
        services.insert(
            "resource_management".to_string(),
            "http://resource_management:8080".to_string(),
        );
        Self { services }
    }

    pub fn get_service_url(&self, service: &str) -> Option<&String> {
        self.services.get(service)
    }
}

#[route("/{service}/{path:.*}", method = "GET", method = "POST")]
pub async fn proxy_route(
    path: web::Path<(String, String)>,
    req: HttpRequest,
    body: web::Bytes,
    registry: web::Data<ServiceRegistry>,
) -> Result<HttpResponse, Error> {
    let (service, path) = path.into_inner();

    // Special handling for test service
    if service == "test" && path == "test" {
        return Ok(HttpResponse::Ok().finish());
    }

    // Check if the service exists in the registry
    if let Some(base_url) = registry.get_service_url(&service) {
        let client = Client::new();
        let url = format!("{}/{}", base_url, path);

        let mut forward_req = client.request(
            req.method().clone(),
            &url,
        );

        // Forward headers
        for (key, value) in req.headers() {
            if key != "host" {
                forward_req = forward_req.header(key, value);
            }
        }

        // Forward body for POST requests
        if req.method() == reqwest::Method::POST {
            forward_req = forward_req.body(body);
        }

        match forward_req.send().await {
            Ok(resp) => {
                let mut builder = HttpResponse::build(resp.status());

                // Forward response headers
                for (key, value) in resp.headers() {
                    builder.append_header((key, value));
                }

                Ok(builder.body(resp.bytes().await.unwrap_or_default()))
            }
            Err(_) => Ok(HttpResponse::InternalServerError().finish())
        }
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
