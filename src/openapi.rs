use utoipa::OpenApi;
use crate::error::ErrorResponse;
use crate::handlers::documents::{CreateDocumentRequest, DocumentResponse};
use crate::handlers::health::HealthResponse;
use crate::handlers::auth::{
    RegisterRequest, RegisterResponse, LoginRequest, LoginResponse,
    RefreshTokenRequest, RefreshTokenResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::health_check,
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh_token,
        crate::handlers::documents::create_document,
        crate::handlers::documents::get_document,
        crate::handlers::documents::list_documents,
    ),
    components(
        schemas(
            ErrorResponse,
            HealthResponse,
            RegisterRequest,
            RegisterResponse,
            LoginRequest,
            LoginResponse,
            RefreshTokenRequest,
            RefreshTokenResponse,
            CreateDocumentRequest,
            DocumentResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "documents", description = "Document management endpoints")
    ),
    security(
        ("jwt_auth" = ["read:documents", "write:documents"])
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "jwt_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    )
                    .bearer_format("JWT"),
                ),
            );
        }
    }
} 