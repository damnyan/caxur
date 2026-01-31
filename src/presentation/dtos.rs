use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::permissions::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDto {
    #[serde(rename = "*")]
    Wildcard,
    AdministratorManagement,
    RoleManagement,
}

impl From<Permission> for PermissionDto {
    fn from(p: Permission) -> Self {
        match p {
            Permission::Wildcard => PermissionDto::Wildcard,
            Permission::AdministratorManagement => PermissionDto::AdministratorManagement,
            Permission::RoleManagement => PermissionDto::RoleManagement,
        }
    }
}

impl From<PermissionDto> for Permission {
    fn from(val: PermissionDto) -> Self {
        match val {
            PermissionDto::Wildcard => Permission::Wildcard,
            PermissionDto::AdministratorManagement => Permission::AdministratorManagement,
            PermissionDto::RoleManagement => Permission::RoleManagement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_dto_conversion() {
        // Test Wildcard conversion
        let domain_wildcard = Permission::Wildcard;
        let dto_wildcard: PermissionDto = domain_wildcard.into();
        assert_eq!(dto_wildcard, PermissionDto::Wildcard);

        let back_wildcard: Permission = dto_wildcard.into();
        assert_eq!(back_wildcard, Permission::Wildcard);

        // Test AdministratorManagement conversion
        let domain_admin = Permission::AdministratorManagement;
        let dto_admin: PermissionDto = domain_admin.into();
        assert_eq!(dto_admin, PermissionDto::AdministratorManagement);

        let back_admin: Permission = dto_admin.into();
        assert_eq!(back_admin, Permission::AdministratorManagement);
    }
}
