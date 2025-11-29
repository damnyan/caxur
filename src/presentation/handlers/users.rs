use crate::application::users::create::{CreateUserRequest, CreateUserUseCase};
use crate::application::users::get::GetUserUseCase;
use crate::application::users::list::{ListUsersRequest, ListUsersUseCase};
use crate::application::users::update::{UpdateUserRequest, UpdateUserUseCase};
use crate::application::users::delete::DeleteUserUseCase;
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::shared::response::ApiResponse;
use crate::shared::validation::ValidatedJson;
use crate::shared::error::AppError;
use crate::infrastructure::db::DbPool;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

pub async fn create_user(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = CreateUserUseCase::new(repo);

    let user = use_case.execute(req).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::new(user))))
}

pub async fn get_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = GetUserUseCase::new(repo);

    let user = use_case.execute(id).await?;

    match user {
        Some(user) => Ok((StatusCode::OK, Json(ApiResponse::new(user)))),
        None => Err(AppError::NotFound),
    }
}

pub async fn list_users(
    State(pool): State<DbPool>,
    Query(req): Query<ListUsersRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = ListUsersUseCase::new(repo);

    let users = use_case.execute(req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(users))))
}

pub async fn update_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = UpdateUserUseCase::new(repo);

    let user = use_case.execute(id, req).await?;

    Ok((StatusCode::OK, Json(ApiResponse::new(user))))
}

pub async fn delete_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
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
