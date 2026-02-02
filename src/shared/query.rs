use crate::shared::error::AppError;
// use async_trait::async_trait; // Removed
use axum::{extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;
use serde_qs::Config;

/// Extractor that deserializes query strings into some type using `serde_qs`.
/// This supports nested query parameters like `page[number]=1`.
pub struct Qs<T>(pub T);

impl<T, S> FromRequestParts<S> for Qs<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        match Config::default().deserialize_str::<T>(query) {
            Ok(value) => Ok(Qs(value)),
            Err(e) => {
                tracing::warn!("Failed to parse query string: {}", e);
                Err(AppError::ValidationError(format!(
                    "Invalid query parameters: {}",
                    e
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestParams {
        page: Page,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Page {
        number: i32,
    }

    #[tokio::test]
    async fn test_qs_valid() {
        let uri = "/?page[number]=1";
        let request = Request::builder().uri(uri).body(()).unwrap();
        let (mut parts, _) = request.into_parts();

        let Qs(params) = Qs::<TestParams>::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(params.page.number, 1);
    }

    #[tokio::test]
    async fn test_qs_invalid() {
        let uri = "/?page[number]=not_a_number";
        let request = Request::builder().uri(uri).body(()).unwrap();
        let (mut parts, _) = request.into_parts();

        let result = Qs::<TestParams>::from_request_parts(&mut parts, &()).await;

        match result {
            Ok(_) => panic!("Should have failed"),
            Err(e) => match e {
                AppError::ValidationError(msg) => {
                    assert!(msg.contains("Invalid query parameters"));
                }
                _ => panic!("Wrong error type"),
            },
        }
    }
}
