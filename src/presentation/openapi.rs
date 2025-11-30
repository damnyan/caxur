#[allow(unused_imports)]
use crate::application::auth::login::{LoginRequest, LoginResponse};
use crate::application::auth::refresh::{RefreshTokenRequest, RefreshTokenResponse};
use crate::application::users::create::CreateUserRequest;
use crate::application::users::list::ListUsersRequest;
use crate::application::users::update::UpdateUserRequest;
use crate::domain::users::User;
use crate::presentation::handlers::auth::AuthTokenResource;
use crate::presentation::handlers::users::UserResource;
use crate::shared::error::{ErrorResponse, JsonApiError, JsonApiErrorSource};
use crate::shared::response::{JsonApiLinks, JsonApiMeta, JsonApiResource, JsonApiResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Caxur User API",
        version = "0.1.0",
        description = "Clean Architecture REST API with Axum and SQLx\n\nThis API follows the JSON:API v1.1 specification for all responses.",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    paths(
        crate::presentation::handlers::auth::login,
        crate::presentation::handlers::auth::refresh_token,
        crate::presentation::handlers::users::create_user,
        crate::presentation::handlers::users::get_user,
        crate::presentation::handlers::users::list_users,
        crate::presentation::handlers::users::update_user,
        crate::presentation::handlers::users::delete_user,
    ),
    components(
        schemas(
            // Domain models
            User,

            // Request DTOs
            CreateUserRequest,
            UpdateUserRequest,
            ListUsersRequest,
            LoginRequest,
            LoginResponse,
            RefreshTokenRequest,
            RefreshTokenResponse,

            // JSON:API Resource types
            UserResource,
            AuthTokenResource,
            JsonApiResource<UserResource>,
            JsonApiResource<AuthTokenResource>,

            // JSON:API Response types
            JsonApiResponse<JsonApiResource<UserResource>>,
            JsonApiResponse<Vec<JsonApiResource<UserResource>>>,
            JsonApiResponse<JsonApiResource<AuthTokenResource>>,
            JsonApiResponse<serde_json::Value>,

            // JSON:API Metadata and Links
            JsonApiMeta,
            JsonApiLinks,

            // JSON:API Error types
            ErrorResponse,
            JsonApiError,
            JsonApiErrorSource,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

use utoipa::Modify;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
