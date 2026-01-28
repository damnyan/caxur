use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Pre-defined permissions for RBAC system
/// These permissions control access to various resources and operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    #[serde(rename = "administrator_management")]
    AdministratorManagement,
    #[serde(rename = "role_management")]
    RoleManagement,
}

impl Permission {
    /// Returns all available permissions
    pub fn all() -> Vec<Permission> {
        vec![
            Permission::AdministratorManagement,
            Permission::RoleManagement,
        ]
    }

    /// Returns a human-readable description of the permission
    pub fn description(&self) -> &str {
        match self {
            Permission::AdministratorManagement => "Manage administrators",
            Permission::RoleManagement => "Manage roles and permissions",
        }
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Permission::AdministratorManagement => "administrator_management",
            Permission::RoleManagement => "role_management",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "administrator_management" => Ok(Permission::AdministratorManagement),
            "role_management" => Ok(Permission::RoleManagement),
            _ => Err(format!("Unknown permission: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_display() {
        assert_eq!(
            Permission::AdministratorManagement.to_string(),
            "administrator_management"
        );
        assert_eq!(Permission::RoleManagement.to_string(), "role_management");
    }

    #[test]
    fn test_permission_from_str() {
        assert_eq!(
            "administrator_management".parse::<Permission>().unwrap(),
            Permission::AdministratorManagement
        );
        assert_eq!(
            "role_management".parse::<Permission>().unwrap(),
            Permission::RoleManagement
        );
        assert!("invalid.permission".parse::<Permission>().is_err());
    }

    #[test]
    fn test_permission_description() {
        assert_eq!(
            Permission::AdministratorManagement.description(),
            "Manage administrators"
        );
        assert_eq!(
            Permission::RoleManagement.description(),
            "Manage roles and permissions"
        );
    }

    #[test]
    fn test_permission_all() {
        let all_permissions = Permission::all();
        assert_eq!(all_permissions.len(), 2);
        assert!(all_permissions.contains(&Permission::AdministratorManagement));
        assert!(all_permissions.contains(&Permission::RoleManagement));
    }

    #[test]
    fn test_permission_serialization() {
        let permission = Permission::AdministratorManagement;
        let json = serde_json::to_string(&permission).unwrap();
        assert_eq!(json, "\"administrator_management\"");

        let deserialized: Permission = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Permission::AdministratorManagement);
    }
}
