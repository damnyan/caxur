#[allow(unused_imports)]
use crate::application::auth::admin_login::AdminLoginRequest;
use crate::application::auth::login::{LoginRequest, LoginResponse};
use crate::application::auth::refresh::{RefreshTokenRequest, RefreshTokenResponse};
use crate::application::roles::create::CreateRoleRequest;
use crate::application::roles::update::UpdateRoleRequest;
use crate::application::users::create::CreateUserRequest;
use crate::application::users::list::ListUsersRequest;
use crate::application::users::update::UpdateUserRequest;
use crate::presentation::admin::handlers::permissions::PermissionResource;
use crate::presentation::admin::handlers::roles::{
    AttachPermissionRequest, DetachPermissionRequest, ListRolesQuery, RoleResource,
};
use crate::presentation::dtos::{AuthTokenResource, PermissionDto, UserResource};
use crate::shared::error::{ErrorResponse, JsonApiError, JsonApiErrorSource};
use crate::shared::response::{JsonApiLinks, JsonApiMeta, JsonApiResource, JsonApiResponse};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Caxur API",
        version = "0.1.0",
        description = "Clean Architecture REST API with Axum and SQLx\n\nThis API follows the JSON:API v1.1 specification for all responses.",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local Development Server"),
    ),
    paths(
        crate::presentation::client::handlers::auth::login,
        crate::presentation::admin::handlers::auth::admin_login,
        crate::presentation::client::handlers::auth::refresh_token,
        crate::presentation::client::handlers::users::create_user,
        crate::presentation::client::handlers::users::get_user,
        crate::presentation::admin::handlers::users::list_users,
        crate::presentation::client::handlers::users::update_user,
        crate::presentation::client::handlers::users::delete_user,
        crate::presentation::admin::handlers::administrators::create_admin,
        crate::presentation::admin::handlers::administrators::get_admin,
        crate::presentation::admin::handlers::administrators::list_admins,
        crate::presentation::admin::handlers::administrators::update_admin,
        crate::presentation::admin::handlers::administrators::delete_admin,
        crate::presentation::admin::handlers::administrators::attach_admin_roles,
        crate::presentation::admin::handlers::administrators::detach_admin_roles,
        crate::presentation::admin::handlers::roles::create_role,
        crate::presentation::admin::handlers::roles::get_role,
        crate::presentation::admin::handlers::roles::list_roles,
        crate::presentation::admin::handlers::roles::update_role,
        crate::presentation::admin::handlers::roles::delete_role,
        crate::presentation::admin::handlers::roles::attach_permission,
        crate::presentation::admin::handlers::roles::detach_permission,
        crate::presentation::admin::handlers::roles::get_role_permissions,
        crate::presentation::admin::handlers::permissions::list_permissions,
    ),
    components(
        schemas(
            // Domain models removed (using Resources/DTOs)
            PermissionDto,

            // Request DTOs
            CreateUserRequest,
            UpdateUserRequest,
            ListUsersRequest,
            CreateRoleRequest,
            UpdateRoleRequest,
            AttachPermissionRequest,
            DetachPermissionRequest,
            ListRolesQuery,
            LoginRequest,

            AdminLoginRequest,
            LoginResponse,
            RefreshTokenRequest,
            RefreshTokenResponse,

            // JSON:API Resource types
            UserResource,
            RoleResource,
            PermissionResource,
            AuthTokenResource,
            JsonApiResource<UserResource>,
            JsonApiResource<RoleResource>,
            JsonApiResource<PermissionResource>,
            JsonApiResource<AuthTokenResource>,

            // JSON:API Response types
            JsonApiResponse<JsonApiResource<UserResource>>,
            JsonApiResponse<Vec<JsonApiResource<UserResource>>>,
            JsonApiResponse<JsonApiResource<RoleResource>>,
            JsonApiResponse<Vec<JsonApiResource<RoleResource>>>,
            JsonApiResponse<Vec<JsonApiResource<PermissionResource>>>,
            JsonApiResponse<Vec<PermissionDto>>,
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
        (name = "Client / Auth", description = "Client Authentication endpoints"),
        (name = "Client / User", description = "User endpoints"),
        (name = "Admin / Auth", description = "Administrator Authentication endpoints"),
        (name = "Admin / Administrator Management", description = "Administrator management endpoints"),
        (name = "Admin / User Management", description = "User management endpoints"),
        (name = "Admin / Role Management", description = "Role management endpoints"),
        (name = "Admin / Permission Management", description = "Permission management endpoints")
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
