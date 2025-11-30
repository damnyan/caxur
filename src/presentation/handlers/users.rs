use crate::application::users::create::{CreateUserRequest, CreateUserUseCase};
use crate::application::users::delete::DeleteUserUseCase;
use crate::application::users::get::GetUserUseCase;
use crate::application::users::list::{ListUsersRequest, ListUsersUseCase};
use crate::application::users::update::{UpdateUserRequest, UpdateUserUseCase};
use crate::domain::users::{User, UserRepository};
use crate::infrastructure::db::DbPool;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::presentation::handlers::auth::AuthUser;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::{JsonApiLinks, JsonApiMeta, JsonApiResource, JsonApiResponse};
use crate::shared::validation::ValidatedJson;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResource {
    pub id: String,
    pub username: String,
    pub email: String,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String)]
    pub created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String)]
    pub updated_at: time::OffsetDateTime,
}

impl From<User> for UserResource {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = JsonApiResponse<JsonApiResource<UserResource>>),
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
        Some(user) => {
            let resource =
                JsonApiResource::new("users", user.id.to_string(), UserResource::from(user));
            Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
        }
        None => Err(AppError::NotFound),
    }
}

/// List all users with pagination
#[utoipa::path(
    get,
    path = "/api/v1/users",
    params(ListUsersRequest),
    responses(
        (status = 200, description = "List of users", body = JsonApiResponse<Vec<JsonApiResource<UserResource>>>),
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
    let use_case = ListUsersUseCase::new(repo.clone());

    // Capture pagination values before moving req
    let page_number = req.page.number;
    let page_size = req.page.size;

    let users = use_case.execute(req).await?;

    // Get total count for pagination
    let total = repo
        .count()
        .await
        .map_err(|e| AppError::InternalServerError(e))?;

    let resources: Vec<JsonApiResource<UserResource>> = users
        .into_iter()
        .map(|user| JsonApiResource::new("users", user.id.to_string(), UserResource::from(user)))
        .collect();

    // Calculate pagination metadata
    let total_pages = (total as f64 / page_size as f64).ceil() as i64;

    let meta = JsonApiMeta::new()
        .with_page(page_number)
        .with_per_page(page_size)
        .with_total(total);

    // Generate pagination links
    let base_url = "/api/v1/users";
    let mut links = JsonApiLinks::new()
        .with_self(format!(
            "{}?page[number]={}&page[size]={}",
            base_url, page_number, page_size
        ))
        .with_first(format!(
            "{}?page[number]=1&page[size]={}",
            base_url, page_size
        ));

    if total_pages > 0 {
        links = links.with_last(format!(
            "{}?page[number]={}&page[size]={}",
            base_url, total_pages, page_size
        ));
    }

    if page_number > 1 {
        links = links.with_prev(format!(
            "{}?page[number]={}&page[size]={}",
            base_url,
            page_number - 1,
            page_size
        ));
    }

    if page_number < total_pages {
        links = links.with_next(format!(
            "{}?page[number]={}&page[size]={}",
            base_url,
            page_number + 1,
            page_size
        ));
    }

    Ok((
        StatusCode::OK,
        Json(
            JsonApiResponse::new(resources)
                .with_meta(meta)
                .with_links(links),
        ),
    ))
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
        let meta = JsonApiMeta::new().with_extra(json!({ "deleted": true }));
        Ok((
            StatusCode::OK,
            Json(JsonApiResponse::new(json!(null)).with_meta(meta)),
        ))
    } else {
        Err(AppError::NotFound)
    }
}
