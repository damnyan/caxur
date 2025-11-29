use crate::application::users::create::{CreateUserRequest, CreateUserUseCase};
use crate::application::users::delete::DeleteUserUseCase;
use crate::application::users::get::GetUserUseCase;
use crate::application::users::list::{ListUsersRequest, ListUsersUseCase};
use crate::application::users::update::{UpdateUserRequest, UpdateUserUseCase};
use crate::domain::users::User;
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::presentation::handlers::auth::AuthUser;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::ApiResponse;
use crate::shared::validation::ValidatedJson;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = ApiResponse<User>),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    tag = "users"
)]
pub async fn create_user(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = CreateUserUseCase::new(repo);

    let user = use_case.execute(req).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::new(user))))
}

/// Get a user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn get_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = GetUserUseCase::new(repo);

    let user = use_case.execute(id).await?;

    match user {
        Some(user) => Ok((StatusCode::OK, Json(ApiResponse::new(user)))),
        None => Err(AppError::NotFound),
    }
}

/// List all users with pagination
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(ListUsersRequest),
    responses(
        (status = 200, description = "List of users", body = ApiResponse<Vec<User>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn list_users(
    State(pool): State<DbPool>,
    Query(req): Query<ListUsersRequest>,
    _auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = ListUsersUseCase::new(repo);

    let users = use_case.execute(req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(users))))
}

/// Update a user
#[utoipa::path(
    put,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Can only update your own account", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn update_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify ownership: user can only update their own account
    let auth_user_id = auth
        .claims
        .user_id()
        .map_err(|e| AppError::InternalServerError(e))?;
    if auth_user_id != id {
        return Err(AppError::Forbidden(
            "You can only update your own account".to_string(),
        ));
    }

    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = UpdateUserUseCase::new(repo);

    let user = use_case.execute(id, req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(user))))
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Can only delete your own account", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // Verify ownership: user can only delete their own account
    let auth_user_id = auth
        .claims
        .user_id()
        .map_err(|e| AppError::InternalServerError(e))?;
    if auth_user_id != id {
        return Err(AppError::Forbidden(
            "You can only delete your own account".to_string(),
        ));
    }

    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = DeleteUserUseCase::new(repo);

    let deleted = use_case.execute(id).await?;

    if deleted {
        Ok((
            StatusCode::OK,
            Json(ApiResponse::new(json!({ "deleted": true }))),
        ))
    } else {
        Err(AppError::NotFound)
    }
}
