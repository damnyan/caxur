use crate::shared::error::{AppError, FieldError};
use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

fn format_validation_error(err: &validator::ValidationError) -> String {
    // 1. Check if a custom message is provided
    if let Some(msg) = &err.message {
        return msg.to_string();
    }

    // 2. Format message based on validation code and parameters
    match err.code.as_ref() {
        "length" => {
            let min = err.params.get("min");
            let max = err.params.get("max");
            let equal = err.params.get("equal");

            if let Some(equal) = equal {
                return format!("Must be exactly {} characters long.", equal);
            }
            if let (Some(min), Some(max)) = (min, max) {
                return format!("Must be between {} and {} characters long.", min, max);
            }
            if let Some(min) = min {
                return format!("Must be at least {} characters long.", min);
            }
            if let Some(max) = max {
                return format!("Must be at most {} characters long.", max);
            }
            "Invalid length.".to_string()
        }
        "range" => {
            let min = err.params.get("min");
            let max = err.params.get("max");

            if let (Some(min), Some(max)) = (min, max) {
                return format!("Must be between {} and {}.", min, max);
            }
            if let Some(min) = min {
                return format!("Must be at least {}.", min);
            }
            if let Some(max) = max {
                return format!("Must be at most {}.", max);
            }
            "Invalid range.".to_string()
        }
        "email" => "Invalid email address.".to_string(),
        "url" => "Invalid URL.".to_string(),
        "credit_card" => "Invalid credit card number.".to_string(),
        "phone" => "Invalid phone number.".to_string(),
        "must_match" => {
            if let Some(other) = err.params.get("other") {
                return format!("Must match the '{}' field.", other);
            }
            "Values do not match.".to_string()
        }
        _ => {
            // Fallback to the code itself if no mapping exists, formatted nicely
            // e.g., "invalid_value" -> "Invalid value"
            let code = err.code.to_string();
            let mut chars = code.chars();
            match chars.next() {
                None => "Invalid value.".to_string(),
                Some(first) => {
                    let rest: String = chars.collect();
                    format!("{}{}", first.to_uppercase(), rest.replace('_', " "))
                }
            }
        }
    }
}

pub fn flatten_validation_errors(e: validator::ValidationErrors) -> Vec<FieldError> {
    e.field_errors()
        .iter()
        .map(|(field, errors)| {
            let message = errors
                .first()
                .map(format_validation_error)
                .unwrap_or_else(|| "Invalid value".to_string());

            FieldError::new(field.to_string(), message)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|e| {
            AppError::ValidationError(vec![FieldError::new("request_body", e.to_string())])
        })?;

        value
            .validate()
            .map_err(|e| AppError::ValidationError(flatten_validation_errors(e)))?;

        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MaybeValidatedJson<T>(pub Option<T>);

impl<T, S> FromRequest<S> for MaybeValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
            AppError::ValidationError(vec![FieldError::new("request_body", e.to_string())])
        })?;

        if bytes.is_empty() {
            return Ok(MaybeValidatedJson(None));
        }

        let new_req = Request::from_parts(parts, axum::body::Body::from(bytes));
        let res = ValidatedJson::<T>::from_request(new_req, state).await?;
        Ok(MaybeValidatedJson(Some(res.0)))
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
            AppError::ValidationError(errors) => {
                let error = errors.iter().find(|e| e.field == "name").unwrap();
                assert_eq!(error.message, "Must be at least 3 characters long.");
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
            AppError::ValidationError(errors) => {
                assert_eq!(errors[0].field, "request_body");
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
