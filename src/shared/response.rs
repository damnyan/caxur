use serde::Serialize;
use utoipa::ToSchema;

/// JSON:API compliant resource wrapper
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiResource<T> {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
    pub attributes: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<std::collections::HashMap<String, JsonApiRelationship>>,
}

#[derive(Serialize, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiRelationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonApiRelationshipData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiLinks>,
}

#[derive(Serialize, ToSchema)]
#[serde(untagged)]
pub enum JsonApiRelationshipData {
    Single(JsonApiIdentifier),
    Many(Vec<JsonApiIdentifier>),
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiIdentifier {
    #[serde(rename = "type")]
    pub resource_type: String,
    pub id: String,
}

/// JSON:API compliant response for single resource or collection
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<JsonApiMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiLinks>,
}

/// JSON:API metadata
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(flatten)]
    pub extra: Option<serde_json::Value>,
}

/// JSON:API links for pagination
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiLinks {
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// Generic collection response for application logic
pub struct CollectionResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Helper for building JSON:API collection responses
#[derive(Serialize, ToSchema)]
pub struct JsonApiCollectionResponse<T>(Vec<JsonApiResource<T>>);

impl<T> JsonApiCollectionResponse<T> {
    pub fn from_domain<D, F>(
        collection: CollectionResponse<D>,
        mapper: F,
    ) -> JsonApiResponse<Vec<JsonApiResource<T>>>
    where
        F: Fn(D) -> JsonApiResource<T>,
        T: Serialize,
    {
        let resources: Vec<JsonApiResource<T>> = collection.data.into_iter().map(mapper).collect();

        let meta = JsonApiMeta::new()
            .with_total(collection.total)
            .with_page(collection.page)
            .with_per_page(collection.per_page);

        JsonApiResponse::new(resources).with_meta(meta)
    }
}

impl<T> JsonApiResponse<T> {
    /// Create a new JSON:API response with data
    pub fn new(data: T) -> Self {
        Self {
            data,
            included: None,
            meta: None,
            links: None,
        }
    }

    /// Add included resources to the response
    pub fn with_included(mut self, included: Vec<serde_json::Value>) -> Self {
        self.included = Some(included);
        self
    }

    /// Add metadata to the response
    pub fn with_meta(mut self, meta: JsonApiMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Add links to the response
    pub fn with_links(mut self, links: JsonApiLinks) -> Self {
        self.links = Some(links);
        self
    }
}

impl<T: Serialize> JsonApiResource<T> {
    /// Create a new JSON:API resource
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>, attributes: T) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: id.into(),
            attributes,
            relationships: None,
        }
    }

    /// Add relationships to the resource
    pub fn with_relationships(
        mut self,
        relationships: std::collections::HashMap<String, JsonApiRelationship>,
    ) -> Self {
        self.relationships = Some(relationships);
        self
    }
}

impl JsonApiRelationship {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_data(mut self, data: JsonApiRelationshipData) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_links(mut self, links: JsonApiLinks) -> Self {
        self.links = Some(links);
        self
    }
}

impl JsonApiIdentifier {
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }
}

impl JsonApiMeta {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            total: None,
            page: None,
            per_page: None,
            extra: None,
        }
    }

    /// Set total count
    pub fn with_total(mut self, total: i64) -> Self {
        self.total = Some(total);
        self
    }

    /// Set page number
    pub fn with_page(mut self, page: i64) -> Self {
        self.page = Some(page);
        self
    }

    /// Set per_page count
    pub fn with_per_page(mut self, per_page: i64) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Add extra metadata
    pub fn with_extra(mut self, extra: serde_json::Value) -> Self {
        self.extra = Some(extra);
        self
    }
}

impl Default for JsonApiMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonApiLinks {
    /// Create new links
    pub fn new() -> Self {
        Self {
            self_link: None,
            first: None,
            last: None,
            prev: None,
            next: None,
        }
    }

    /// Set self link
    pub fn with_self(mut self, self_link: impl Into<String>) -> Self {
        self.self_link = Some(self_link.into());
        self
    }

    /// Set first link
    pub fn with_first(mut self, first: impl Into<String>) -> Self {
        self.first = Some(first.into());
        self
    }

    /// Set last link
    pub fn with_last(mut self, last: impl Into<String>) -> Self {
        self.last = Some(last.into());
        self
    }

    /// Set prev link
    pub fn with_prev(mut self, prev: impl Into<String>) -> Self {
        self.prev = Some(prev.into());
        self
    }

    /// Set next link
    pub fn with_next(mut self, next: impl Into<String>) -> Self {
        self.next = Some(next.into());
        self
    }
}

impl Default for JsonApiLinks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_api_resource() {
        let resource = JsonApiResource::new("users", "123", json!({"name": "John"}));

        assert_eq!(resource.resource_type, "users");
        assert_eq!(resource.id, "123");
        assert_eq!(resource.attributes, json!({"name": "John"}));
    }

    #[test]
    fn test_json_api_response() {
        let resource = JsonApiResource::new("users", "123", json!({"name": "John"}));
        let response = JsonApiResponse::new(resource);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["data"]["type"], "users");
        assert_eq!(json["data"]["id"], "123");
        assert_eq!(json["data"]["attributes"]["name"], "John");
    }

    #[test]
    fn test_json_api_response_with_meta() {
        let resource = JsonApiResource::new("users", "123", json!({"name": "John"}));
        let meta = JsonApiMeta::new().with_total(100).with_page(1);
        let response = JsonApiResponse::new(resource).with_meta(meta);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["meta"]["total"], 100);
        assert_eq!(json["meta"]["page"], 1);
    }

    #[test]
    fn test_json_api_response_with_links() {
        let resource = JsonApiResource::new("users", "123", json!({"name": "John"}));
        let links = JsonApiLinks::new()
            .with_self("/api/v1/users/123")
            .with_first("/api/v1/users?page=1");
        let response = JsonApiResponse::new(resource).with_links(links);

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["links"]["self"], "/api/v1/users/123");
        assert_eq!(json["links"]["first"], "/api/v1/users?page=1");
    }

    #[test]
    fn test_json_api_collection_response() {
        let resources = vec![
            JsonApiResource::new("users", "1", json!({"name": "John"})),
            JsonApiResource::new("users", "2", json!({"name": "Jane"})),
        ];
        let meta = JsonApiMeta::new().with_total(2);
        let response = JsonApiResponse::new(resources).with_meta(meta);

        let json = serde_json::to_value(&response).unwrap();
        assert!(json["data"].is_array());
        assert_eq!(json["data"].as_array().unwrap().len(), 2);
        assert_eq!(json["meta"]["total"], 2);
    }

    #[test]
    fn test_json_api_meta_with_per_page() {
        let meta = JsonApiMeta::new().with_per_page(20);

        assert_eq!(meta.per_page, Some(20));
    }

    #[test]
    fn test_json_api_meta_with_extra() {
        let meta = JsonApiMeta::new().with_extra(json!({"custom": "value"}));

        assert_eq!(meta.extra, Some(json!({"custom": "value"})));
    }

    #[test]
    fn test_json_api_meta_default() {
        let meta = JsonApiMeta::default();

        assert!(meta.page.is_none());
        assert!(meta.per_page.is_none());
        assert!(meta.total.is_none());
        assert!(meta.extra.is_none());
    }

    #[test]
    fn test_json_api_links_with_last() {
        let links = JsonApiLinks::new().with_last("/api/v1/users?page=10");

        assert_eq!(links.last, Some("/api/v1/users?page=10".to_string()));
    }

    #[test]
    fn test_json_api_links_with_prev() {
        let links = JsonApiLinks::new().with_prev("/api/v1/users?page=1");

        assert_eq!(links.prev, Some("/api/v1/users?page=1".to_string()));
    }

    #[test]
    fn test_json_api_links_with_next() {
        let links = JsonApiLinks::new().with_next("/api/v1/users?page=3");

        assert_eq!(links.next, Some("/api/v1/users?page=3".to_string()));
    }

    #[test]
    fn test_json_api_links_default() {
        let links = JsonApiLinks::default();

        assert!(links.self_link.is_none());
        assert!(links.first.is_none());
        assert!(links.last.is_none());
        assert!(links.prev.is_none());
        assert!(links.next.is_none());
    }
}
