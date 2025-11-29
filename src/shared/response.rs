use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    pub fn with_meta(mut self, meta: serde_json::Value) -> Self {
        self.meta = Some(meta);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_api_response_new() {
        let data = "test data";
        let response = ApiResponse::new(data);

        assert_eq!(response.data, "test data");
        assert!(response.meta.is_none());
    }

    #[test]
    fn test_api_response_with_meta() {
        let data = "test data";
        let meta = json!({"page": 1, "total": 10});
        let response = ApiResponse::new(data).with_meta(meta.clone());

        assert_eq!(response.data, "test data");
        assert_eq!(response.meta, Some(meta));
    }

    #[test]
    fn test_api_response_serialization() {
        let data = "test data";
        let meta = json!({"page": 1});
        let response = ApiResponse::new(data).with_meta(meta);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"], "test data");
        assert_eq!(json["meta"]["page"], 1);
    }
}
