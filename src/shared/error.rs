use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
    #[error("Internal server error: {0}")]
    InternalServerError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ValidationError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = json!({
            "errors": [
                {
                    "status": status.as_u16(),
                    "detail": message
                }
            ]
        });

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_validation_error_response() {
        let err = AppError::ValidationError("Invalid input".to_string());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["errors"][0]["status"], 422);
        assert_eq!(body_json["errors"][0]["detail"], "Invalid input");
    }

    #[tokio::test]
    async fn test_not_found_error_response() {
        let err = AppError::NotFound;
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["errors"][0]["status"], 404);
        assert_eq!(body_json["errors"][0]["detail"], "Resource not found");
    }
}
