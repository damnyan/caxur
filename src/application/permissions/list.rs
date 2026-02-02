use crate::domain::permissions::{Permission, PermissionScope};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
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
    pub fn execute(&self) -> Vec<PermissionResponse> {
        Permission::all()
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
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_permissions_no_scope() {
        let use_case = ListPermissionsUseCase::new();
        let permissions = use_case.execute();

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
        let permissions = use_case.execute();

        // Since all permissions currently have ADMINISTRATOR scope
        assert_eq!(permissions.len(), 3);
        assert!(
            permissions
                .iter()
                .any(|p| p.name == "administrator_management")
        );
    }

    #[test]
    fn test_permission_response_structure() {
        let use_case = ListPermissionsUseCase::new();
        let permissions = use_case.execute();

        let admin_mgmt = permissions
            .iter()
            .find(|p| p.name == "administrator_management")
            .unwrap();
        assert_eq!(admin_mgmt.description, "Manage administrators");
    }
}
