use crate::shared::response::JsonApiLinks;
use axum::http::Uri;

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
