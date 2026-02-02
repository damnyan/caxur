use crate::domain::permissions::{Permission, PermissionScope};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Debug)]
pub struct PageParams {
    /// Page number (1-indexed)
    #[serde(default = "crate::shared::pagination::default_page_number")]
    #[param(example = 1, minimum = 1)]
    pub number: i64,
    /// Number of items per page
    #[serde(default = "crate::shared::pagination::default_page_size")]
    #[param(example = 20, minimum = 1, maximum = 100)]
    pub size: i64,
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            number: crate::shared::pagination::default_page_number(),
            size: crate::shared::pagination::default_page_size(),
        }
    }
}

#[derive(serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema, Debug)]
pub struct ListPermissionsRequest {
    /// Pagination parameters
    #[serde(default)]
    pub page: PageParams,
}

#[derive(Serialize, ToSchema, Clone)]
pub struct PermissionResponse {
    #[schema(example = "users.create")]
    pub name: String,
    #[schema(example = "Create new users")]
    pub description: String,
}

#[derive(Default)]
pub struct ListPermissionsUseCase {
    scope: Option<PermissionScope>,
}

impl ListPermissionsUseCase {
    pub fn new() -> Self {
        Self { scope: None }
    }

    pub fn with_scope(mut self, scope: impl Into<PermissionScope>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    #[tracing::instrument(skip(self))]
    pub fn execute(&self, req: ListPermissionsRequest) -> Vec<PermissionResponse> {
        let all_permissions: Vec<PermissionResponse> = Permission::all()
            .into_iter()
            .filter(|p| {
                if let Some(scope) = &self.scope {
                    p.scopes().contains(scope)
                } else {
                    true
                }
            })
            .map(|p| PermissionResponse {
                name: p.to_string(),
                description: p.description().to_string(),
            })
            .collect();

        // Pagination logic
        let total = all_permissions.len() as i64;
        let per_page = req.page.size.clamp(1, 100);
        let page = req.page.number.max(1);
        let offset = ((page - 1) * per_page) as usize;

        if offset >= total as usize {
            return Vec::new();
        }

        let end = (offset + per_page as usize).min(total as usize);
        all_permissions[offset..end].to_vec()
    }

    /// Get total count for metadata
    pub fn count(&self) -> i64 {
        Permission::all()
            .into_iter()
            .filter(|p| {
                if let Some(scope) = &self.scope {
                    p.scopes().contains(scope)
                } else {
                    true
                }
            })
            .count() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_permissions_no_scope() {
        let use_case = ListPermissionsUseCase::new();
        let req = ListPermissionsRequest {
            page: PageParams::default(),
        };
        let permissions = use_case.execute(req);

        assert_eq!(permissions.len(), 3);
        assert!(
            permissions
                .iter()
                .any(|p| p.name == "administrator_management")
        );
        assert!(permissions.iter().any(|p| p.name == "role_management"));
        assert!(permissions.iter().any(|p| p.name == "*"));
    }

    #[test]
    fn test_list_permissions_with_admin_scope() {
        let use_case = ListPermissionsUseCase::new().with_scope(PermissionScope::Administrator);
        let req = ListPermissionsRequest {
            page: PageParams::default(),
        };
        let permissions = use_case.execute(req);

        // Since all permissions currently have ADMINISTRATOR scope
        assert_eq!(permissions.len(), 3);
        assert!(
            permissions
                .iter()
                .any(|p| p.name == "administrator_management")
        );
    }

    #[test]
    fn test_list_permissions_pagination() {
        let use_case = ListPermissionsUseCase::new();

        // Page 1, size 1
        let req1 = ListPermissionsRequest {
            page: PageParams { number: 1, size: 1 },
        };
        let permissions1 = use_case.execute(req1);
        assert_eq!(permissions1.len(), 1);

        // Page 2, size 1
        let req2 = ListPermissionsRequest {
            page: PageParams { number: 2, size: 1 },
        };
        let permissions2 = use_case.execute(req2);
        assert_eq!(permissions2.len(), 1);
        assert_ne!(permissions1[0].name, permissions2[0].name);

        // Page 10, size 10 (empty)
        let req_empty = ListPermissionsRequest {
            page: PageParams {
                number: 10,
                size: 10,
            },
        };
        let permissions_empty = use_case.execute(req_empty);
        assert!(permissions_empty.is_empty());
    }

    #[test]
    fn test_permission_response_structure() {
        let use_case = ListPermissionsUseCase::new();
        let req = ListPermissionsRequest {
            page: PageParams::default(),
        };
        let permissions = use_case.execute(req);

        let admin_mgmt = permissions
            .iter()
            .find(|p| p.name == "administrator_management")
            .unwrap();
        assert_eq!(admin_mgmt.description, "Manage administrators");
    }

    #[test]
    fn test_count_no_scope() {
        let use_case = ListPermissionsUseCase::new();
        assert_eq!(use_case.count(), 3);
    }

    #[test]
    fn test_count_with_scope() {
        let use_case = ListPermissionsUseCase::new().with_scope(PermissionScope::Administrator);
        assert_eq!(use_case.count(), 3);
    }
}
