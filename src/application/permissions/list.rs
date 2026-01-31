use crate::domain::permissions::Permission;
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
pub struct ListPermissionsUseCase;

impl ListPermissionsUseCase {
    pub fn new() -> Self {
        Self
    }

    #[tracing::instrument(skip(self))]
    pub fn execute(&self) -> Vec<PermissionResponse> {
        Permission::all()
            .into_iter()
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
    fn test_list_permissions() {
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
