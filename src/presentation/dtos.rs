use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::permissions::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDto {
    AdministratorManagement,
    RoleManagement,
}

impl From<Permission> for PermissionDto {
    fn from(p: Permission) -> Self {
        match p {
            Permission::AdministratorManagement => PermissionDto::AdministratorManagement,
            Permission::RoleManagement => PermissionDto::RoleManagement,
        }
    }
}

impl From<PermissionDto> for Permission {
    fn from(val: PermissionDto) -> Self {
        match val {
            PermissionDto::AdministratorManagement => Permission::AdministratorManagement,
            PermissionDto::RoleManagement => Permission::RoleManagement,
        }
    }
}
