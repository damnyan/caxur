use crate::application::users::create::{CreateUserRequest, CreateUserUseCase};
use crate::infrastructure::repositories::users::PostgresUserRepository;
use crate::shared::response::ApiResponse;
use crate::shared::validation::ValidatedJson;
use crate::shared::error::AppError;
use crate::infrastructure::db::DbPool;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn create_user(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = CreateUserUseCase::new(repo);

    let user = use_case.execute(req).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::new(user))))
}
