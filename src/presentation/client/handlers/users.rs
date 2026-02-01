use crate::application::users::create::{CreateUserRequest, CreateUserUseCase};
use crate::application::users::delete::DeleteUserUseCase;
use crate::application::users::get::GetUserUseCase;
use crate::application::users::update::{UpdateUserRequest, UpdateUserUseCase};
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::presentation::dtos::UserResource;
use crate::presentation::extractors::AuthUser;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::{JsonApiMeta, JsonApiResource, JsonApiResponse};
use crate::shared::validation::ValidatedJson;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::infrastructure::password::PasswordService;

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = JsonApiResponse<JsonApiResource<UserResource>>),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Client / User"
)]
pub async fn create_user(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let hasher = Arc::new(PasswordService::new());
    let use_case = CreateUserUseCase::new(repo, hasher);

    let user = use_case.execute(req).await?;
    let resource = JsonApiResource::new("users", user.id.to_string(), UserResource::from(user));

    Ok((StatusCode::CREATED, Json(JsonApiResponse::new(resource))))
}

/// Get a user by ID
#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = JsonApiResponse<JsonApiResource<UserResource>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Client / User"
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
        Some(user) => {
            let resource =
                JsonApiResource::new("users", user.id.to_string(), UserResource::from(user));
            Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
        }
        None => Err(AppError::NotFound("User not found".to_string())),
    }
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
        (status = 200, description = "User updated successfully", body = JsonApiResponse<JsonApiResource<UserResource>>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Can only update your own account", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Client / User"
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
        .map_err(AppError::InternalServerError)?;
    if auth_user_id != id {
        return Err(AppError::Forbidden(
            "You can only update your own account".to_string(),
        ));
    }

    let repo = Arc::new(PostgresUserRepository::new(pool));
    let hasher = Arc::new(PasswordService::new());
    let use_case = UpdateUserUseCase::new(repo, hasher);

    let user = use_case.execute(id, req).await?;
    let resource = JsonApiResource::new("users", user.id.to_string(), UserResource::from(user));

    Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully", body = JsonApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Can only delete your own account", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Client / User"
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
        .map_err(AppError::InternalServerError)?;
    if auth_user_id != id {
        return Err(AppError::Forbidden(
            "You can only delete your own account".to_string(),
        ));
    }

    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = DeleteUserUseCase::new(repo);

    let deleted = use_case.execute(id).await?;

    if deleted {
        let meta = JsonApiMeta::new().with_extra(json!({ "deleted": true }));
        Ok((
            StatusCode::OK,
            Json(JsonApiResponse::new(json!(null)).with_meta(meta)),
        ))
    } else {
        Err(AppError::NotFound("User not found".to_string()))
    }
}
