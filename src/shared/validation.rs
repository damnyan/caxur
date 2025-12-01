use crate::shared::error::AppError;
use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);


impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        value.validate().map_err(|e| {
            // Convert validation errors to a string or structured format
            // For simplicity, we just dump the error here, but in a real app
            // you'd want to format this to match the JSON:API error object structure more closely
            AppError::ValidationError(e.to_string())
        })?;

        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestData {
        #[validate(length(min = 3))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[tokio::test]
    async fn test_validated_json_success() {
        let json_body = r#"{"name": "John", "email": "john@example.com"}"#;
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(req, &()).await;
        assert!(result.is_ok());

        let ValidatedJson(data) = result.unwrap();
        assert_eq!(data.name, "John");
        assert_eq!(data.email, "john@example.com");
    }

    #[tokio::test]
    async fn test_validated_json_validation_error() {
        let json_body = r#"{"name": "Jo", "email": "john@example.com"}"#;
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(req, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::ValidationError(msg) => {
                assert!(msg.contains("name"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_validated_json_parse_error() {
        let json_body = r#"{"invalid json"#;
        let req = Request::builder()
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap();

        let result = ValidatedJson::<TestData>::from_request(req, &()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            AppError::ValidationError(_) => {}
            _ => panic!("Expected ValidationError"),
        }
    }
}
