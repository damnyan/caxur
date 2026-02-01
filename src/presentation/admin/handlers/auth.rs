use crate::application::auth::admin_login::{AdminLoginRequest, AdminLoginUseCase};
use crate::infrastructure::repositories::administrators::PostgresAdministratorRepository;
use crate::infrastructure::repositories::refresh_tokens::PostgresRefreshTokenRepository;
use crate::infrastructure::state::AppState;
use crate::presentation::dtos::AuthTokenResource;
use crate::shared::error::{AppError, ErrorResponse};
use crate::shared::response::{JsonApiResource, JsonApiResponse};
use crate::shared::validation::ValidatedJson;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

/// Admin Login handler
#[utoipa::path(
    post,
    path = "/api/v1/admin/auth/login",
    request_body = AdminLoginRequest,
    responses(
        (status = 200, description = "Admin Login successful", body = JsonApiResponse<JsonApiResource<AuthTokenResource>>),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse)
    ),
    tag = "Admin / Auth"
)]
pub async fn admin_login(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<AdminLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_service = state.auth_service;
    let pool = state.pool;

    let admin_repo = Arc::new(PostgresAdministratorRepository::new(pool.clone()));
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new(pool));
    let password_service = Arc::new(crate::infrastructure::password::PasswordService::new());

    let access_token_expiry = std::env::var("JWT_ACCESS_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "900".to_string())
        .parse::<i64>()
        .unwrap_or(900);
    let refresh_token_expiry = std::env::var("JWT_REFRESH_TOKEN_EXPIRY")
        .unwrap_or_else(|_| "604800".to_string())
        .parse::<i64>()
        .unwrap_or(604800);

    let use_case = AdminLoginUseCase::new(
        admin_repo,
        refresh_token_repo,
        auth_service,
        password_service,
        access_token_expiry,
        refresh_token_expiry,
    );

    let response = use_case.execute(req).await?;
    let resource =
        JsonApiResource::new("auth-tokens", "session", AuthTokenResource::from(response));

    Ok((StatusCode::OK, Json(JsonApiResponse::new(resource))))
}
