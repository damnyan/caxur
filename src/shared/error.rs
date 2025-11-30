use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// JSON:API compliant error response
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub errors: Vec<JsonApiError>,
}

/// JSON:API error object
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub title: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<JsonApiErrorSource>,
}

/// JSON:API error source
#[derive(Serialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
}

impl JsonApiError {
    /// Create a new JSON:API error
    pub fn new(status: StatusCode, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: None,
            status: status.as_u16().to_string(),
            code: None,
            title: title.into(),
            detail: detail.into(),
            source: None,
        }
    }

    /// Set error ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set error code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set error source
    pub fn with_source(mut self, source: JsonApiErrorSource) -> Self {
        self.source = Some(source);
        self
    }
}

impl JsonApiErrorSource {
    /// Create error source with pointer
    pub fn pointer(pointer: impl Into<String>) -> Self {
        Self {
            pointer: Some(pointer.into()),
            parameter: None,
        }
    }

    /// Create error source with parameter
    pub fn parameter(parameter: impl Into<String>) -> Self {
        Self {
            pointer: None,
            parameter: Some(parameter.into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Internal server error: {0}")]
    InternalServerError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, title, detail, code) = match &self {
            AppError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error",
                msg.clone(),
                Some("validation_error"),
            ),
            AppError::DatabaseError(e) => {
                // Check for unique constraint violations
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        let msg = if db_err.message().contains("username") {
                            "Username already exists"
                        } else if db_err.message().contains("email") {
                            "Email already exists"
                        } else {
                            "Resource already exists"
                        };

                        let error = JsonApiError::new(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            "Unique Constraint Violation",
                            msg,
                        )
                        .with_code("unique_violation");

                        return (
                            StatusCode::UNPROCESSABLE_ENTITY,
                            Json(ErrorResponse {
                                errors: vec![error],
                            }),
                        )
                            .into_response();
                    }
                }
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error",
                    "An error occurred while processing your request".to_string(),
                    Some("database_error"),
                )
            }
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "Not Found",
                "The requested resource was not found".to_string(),
                Some("not_found"),
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "Conflict",
                msg.clone(),
                Some("conflict"),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                msg.clone(),
                Some("unauthorized"),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "Forbidden",
                msg.clone(),
                Some("forbidden"),
            ),
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    "An unexpected error occurred".to_string(),
                    Some("internal_error"),
                )
            }
        };

        let mut error = JsonApiError::new(status, title, detail);
        if let Some(code) = code {
            error = error.with_code(code);
        }

        let body = ErrorResponse {
            errors: vec![error],
        };

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

        assert_eq!(body_json["errors"][0]["status"], "422");
        assert_eq!(body_json["errors"][0]["title"], "Validation Error");
        assert_eq!(body_json["errors"][0]["detail"], "Invalid input");
        assert_eq!(body_json["errors"][0]["code"], "validation_error");
    }

    #[tokio::test]
    async fn test_not_found_error_response() {
        let err = AppError::NotFound;
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["errors"][0]["status"], "404");
        assert_eq!(body_json["errors"][0]["title"], "Not Found");
        assert_eq!(
            body_json["errors"][0]["detail"],
            "The requested resource was not found"
        );
        assert_eq!(body_json["errors"][0]["code"], "not_found");
    }

    #[tokio::test]
    async fn test_unauthorized_error_response() {
        let err = AppError::Unauthorized("Invalid token".to_string());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body_json["errors"][0]["status"], "401");
        assert_eq!(body_json["errors"][0]["title"], "Unauthorized");
        assert_eq!(body_json["errors"][0]["detail"], "Invalid token");
        assert_eq!(body_json["errors"][0]["code"], "unauthorized");
    }

    #[test]
    fn test_json_api_error_builder() {
        let error = JsonApiError::new(StatusCode::BAD_REQUEST, "Bad Request", "Invalid data")
            .with_id("err-123")
            .with_code("bad_request")
            .with_source(JsonApiErrorSource::pointer("/data/attributes/email"));

        assert_eq!(error.id, Some("err-123".to_string()));
        assert_eq!(error.status, "400");
        assert_eq!(error.code, Some("bad_request".to_string()));
        assert_eq!(error.title, "Bad Request");
        assert_eq!(error.detail, "Invalid data");
        assert!(error.source.is_some());
    }
}
