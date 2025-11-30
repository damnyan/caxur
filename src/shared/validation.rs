use crate::shared::error::AppError;
use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

use async_trait::async_trait;

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
