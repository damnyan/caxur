use crate::shared::response::JsonApiLinks;
use axum::http::Uri;

/// Default page number for pagination
pub fn default_page_number() -> i64 {
    1
}

/// Default items per page for pagination
pub fn default_page_size() -> i64 {
    20
}

/// Default sort field for pagination
pub fn default_sort() -> String {
    "created_at".to_string()
}

/// Shared page parameters for pagination
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Debug, Clone)]
pub struct PageParams {
    /// Page number (1-indexed)
    #[serde(
        default = "default_page_number",
        deserialize_with = "deserialize_i64_from_string"
    )]
    #[param(example = 1, minimum = 1, default = 1)]
    #[schema(example = 1, minimum = 1, default = 1)]
    pub number: i64,
    /// Number of items per page
    #[serde(
        default = "default_page_size",
        deserialize_with = "deserialize_i64_from_string"
    )]
    #[param(example = 20, minimum = 1, maximum = 100, default = 20)]
    #[schema(example = 20, minimum = 1, maximum = 100, default = 20)]
    pub size: i64,
}

pub fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => s.parse::<i64>().map_err(serde::de::Error::custom),
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("Invalid number")),
        _ => Err(serde::de::Error::custom("Expected string or number")),
    }
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            number: default_page_number(),
            size: default_page_size(),
        }
    }
}

/// Shared pagination request with page and sort parameters
#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Debug, Clone)]
#[into_params(parameter_in = Query)]
pub struct PaginationRequest {
    /// Pagination parameters
    #[serde(default)]
    #[param(style = DeepObject, explode = true)]
    pub page: PageParams,
    /// Sort fields (comma-separated, prefix with - for descending)
    /// Example: "created_at" or "-created_at,username"
    #[serde(default = "default_sort")]
    #[param(example = "created_at", default = "created_at", required = false)]
    pub sort: String,
    /// Include related resources (comma-separated)
    /// Example: "roles"
    #[serde(default)]
    #[param(required = false)]
    pub include: Option<String>,
}

impl PaginationRequest {
    /// Get the sort field (strips leading -)
    pub fn sort_field(&self) -> &str {
        if self.sort.starts_with('-') {
            &self.sort[1..]
        } else {
            &self.sort
        }
    }

    /// Check if sorting is descending (starts with -)
    pub fn is_descending(&self) -> bool {
        self.sort.starts_with('-')
    }
}

/// Pagination link builder that generates JSON:API compliant pagination links
pub struct PaginationLinkBuilder {
    base_url: String,
    page_number: i64,
    page_size: i64,
    total_pages: i64,
}

impl PaginationLinkBuilder {
    /// Create a new pagination link builder from a URI
    /// Automatically extracts the path from the URI
    pub fn from_uri(uri: &Uri, page_number: i64, page_size: i64, total: i64) -> Self {
        let base_url = uri.path().to_string();
        let total_pages = if total > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i64
        } else {
            0
        };

        Self {
            base_url,
            page_number,
            page_size,
            total_pages,
        }
    }

    /// Create a new pagination link builder with a custom base URL
    pub fn new(base_url: impl Into<String>, page_number: i64, page_size: i64, total: i64) -> Self {
        let total_pages = if total > 0 {
            ((total as f64) / (page_size as f64)).ceil() as i64
        } else {
            0
        };

        Self {
            base_url: base_url.into(),
            page_number,
            page_size,
            total_pages,
        }
    }

    /// Build the pagination links
    pub fn build(self) -> JsonApiLinks {
        let mut links = JsonApiLinks::new()
            .with_self(self.page_link(self.page_number))
            .with_first(self.page_link(1));

        if self.total_pages > 0 {
            links = links.with_last(self.page_link(self.total_pages));
        }

        if self.page_number > 1 {
            links = links.with_prev(self.page_link(self.page_number - 1));
        }

        if self.page_number < self.total_pages {
            links = links.with_next(self.page_link(self.page_number + 1));
        }

        links
    }

    /// Generate a link for a specific page
    fn page_link(&self, page: i64) -> String {
        format!(
            "{}?page[number]={}&page[size]={}",
            self.base_url, page, self.page_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(default_page_number(), 1);
        assert_eq!(default_page_size(), 20);
    }

    #[test]
    fn test_pagination_links_first_page() {
        let builder = PaginationLinkBuilder::new("/api/v1/users", 1, 10, 25);
        let links = builder.build();

        assert_eq!(
            links.self_link,
            Some("/api/v1/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(
            links.first,
            Some("/api/v1/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(
            links.last,
            Some("/api/v1/users?page[number]=3&page[size]=10".to_string())
        );
        assert_eq!(links.prev, None);
        assert_eq!(
            links.next,
            Some("/api/v1/users?page[number]=2&page[size]=10".to_string())
        );
    }

    #[test]
    fn test_pagination_links_middle_page() {
        let builder = PaginationLinkBuilder::new("/api/v1/users", 2, 10, 50);
        let links = builder.build();

        assert_eq!(
            links.self_link,
            Some("/api/v1/users?page[number]=2&page[size]=10".to_string())
        );
        assert_eq!(
            links.prev,
            Some("/api/v1/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(
            links.next,
            Some("/api/v1/users?page[number]=3&page[size]=10".to_string())
        );
    }

    #[test]
    fn test_pagination_links_last_page() {
        let builder = PaginationLinkBuilder::new("/api/v1/users", 3, 10, 25);
        let links = builder.build();

        assert_eq!(
            links.prev,
            Some("/api/v1/users?page[number]=2&page[size]=10".to_string())
        );
        assert_eq!(links.next, None);
    }

    #[test]
    fn test_pagination_links_empty_results() {
        let builder = PaginationLinkBuilder::new("/api/v1/users", 1, 10, 0);
        let links = builder.build();

        assert_eq!(
            links.self_link,
            Some("/api/v1/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(links.last, None);
        assert_eq!(links.prev, None);
        assert_eq!(links.next, None);
    }

    #[test]
    fn test_from_uri() {
        let uri: Uri = "/api/v1/users?page[number]=2&page[size]=10"
            .parse()
            .unwrap();
        let builder = PaginationLinkBuilder::from_uri(&uri, 2, 10, 50);
        let links = builder.build();

        assert_eq!(
            links.self_link,
            Some("/api/v1/users?page[number]=2&page[size]=10".to_string())
        );
    }

    #[test]
    fn test_from_uri_empty() {
        let uri: Uri = "/api/v1/users?page[number]=1&page[size]=10"
            .parse()
            .unwrap();
        let builder = PaginationLinkBuilder::from_uri(&uri, 1, 10, 0);
        let links = builder.build();

        assert_eq!(
            links.self_link,
            Some("/api/v1/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(links.last, None);
    }
}
