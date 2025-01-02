use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use awc::Client;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::env;

#[derive(Debug, Serialize)]
struct ContentSafetyRequest {
    text: Content,
}

#[derive(Debug, Serialize)]
struct Content {
    text: String,
}

#[derive(Debug, Deserialize)]
struct ContentSafetyResponse {
    toxicity: Vec<ToxicityResult>,
}

#[derive(Debug, Deserialize)]
struct ToxicityResult {
    score: f32,
    category: String,
}

pub struct ContentModeratorMiddleware;

impl ContentModeratorMiddleware {
    pub fn new() -> Self {
        ContentModeratorMiddleware
    }

    async fn check_content_safety(text: &str) -> Result<bool, Error> {
        let project_id = env::var("GOOGLE_CLOUD_PROJECT")
            .map_err(|_| actix_web::error::ErrorInternalServerError("Missing project ID"))?;

        let client = Client::default();
        let url = format!(
            "https://contentthreat.googleapis.com/v1beta1/projects/{}/locations/global:analyzeThreat",
            project_id
        );

        let request = ContentSafetyRequest {
            text: Content {
                text: text.to_string(),
            },
        };

        let response = client
            .post(&url)
            .bearer_auth(env::var("GOOGLE_APPLICATION_CREDENTIALS").map_err(|_| {
                actix_web::error::ErrorInternalServerError("Missing credentials")
            })?)
            .send_json(&request)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let safety_response: ContentSafetyResponse = response
            .json()
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // Check if any toxicity score is above threshold (e.g., 0.7)
        let is_toxic = safety_response
            .toxicity
            .iter()
            .any(|result| result.score > 0.7);

        Ok(!is_toxic)
    }
}

impl<S, B> Transform<S, ServiceRequest> for ContentModeratorMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ContentModeratorMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ContentModeratorMiddlewareInner { service }))
    }
}

pub struct ContentModeratorMiddlewareInner<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContentModeratorMiddlewareInner<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let future = self.service.call(req);
        Box::pin(async move {
            let mut response = future.await?;
            
            // Extract text content from response body if needed
            // This is a simplified example - you'll need to implement proper body extraction
            if let Some(content) = response.response().body().as_ref() {
                if let Ok(text) = std::str::from_utf8(content) {
                    if !ContentModeratorMiddleware::check_content_safety(text).await? {
                        return Err(actix_web::error::ErrorForbidden("Content violates safety policy"));
                    }
                }
            }
            
            Ok(response)
        })
    }
}
