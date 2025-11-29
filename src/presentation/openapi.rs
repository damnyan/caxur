use crate::application::users::create::CreateUserRequest;
use crate::application::users::list::ListUsersRequest;
use crate::application::users::update::UpdateUserRequest;
use crate::domain::users::User;
use crate::shared::error::{ErrorDetail, ErrorResponse};
use crate::shared::response::ApiResponse;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Caxur User API",
        version = "0.1.0",
        description = "Clean Architecture REST API with Axum and SQLx",
        contact(
            name = "API Support",
            email = "support@example.com"
        )
    ),
    paths(
        crate::presentation::handlers::users::create_user,
        crate::presentation::handlers::users::get_user,
        crate::presentation::handlers::users::list_users,
        crate::presentation::handlers::users::update_user,
        crate::presentation::handlers::users::delete_user,
    ),
    components(
        schemas(
            User,
            CreateUserRequest,
            UpdateUserRequest,
            ListUsersRequest,
            ApiResponse<User>,
            ApiResponse<Vec<User>>,
            ErrorResponse,
            ErrorDetail,
        )
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;
