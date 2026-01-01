use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::roles::attach_permission::AttachPermissionUseCase;
use caxur::application::roles::get_permissions::GetRolePermissionsUseCase;
use caxur::domain::permissions::Permission;
use caxur::domain::roles::RoleRepository;
use caxur::infrastructure::repositories::roles::PostgresRoleRepository;
use caxur::shared::error::AppError;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_get_permissions_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let attach_use_case = AttachPermissionUseCase::new(repo.clone());
    let get_use_case = GetRolePermissionsUseCase::new(repo.clone());

    let prefix = Uuid::new_v4().to_string();
    let role = repo
        .create(caxur::domain::roles::NewRole {
            name: format!("role_get_{}", prefix),
            description: None,
        })
        .await
        .expect("Failed to create role");

    let permissions = vec![
        Permission::AdministratorManagement,
        Permission::RoleManagement,
    ];
    attach_use_case
        .execute(role.id, permissions.clone())
        .await
        .unwrap();

    let retrieved_permissions = get_use_case
        .execute(role.id)
        .await
        .expect("Failed to get permissions");

    assert!(retrieved_permissions.contains(&Permission::AdministratorManagement));
    assert!(retrieved_permissions.contains(&Permission::RoleManagement));
    assert_eq!(retrieved_permissions.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_get_permissions_role_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = GetRolePermissionsUseCase::new(repo);

    let result = use_case.execute(Uuid::new_v4()).await;

    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}
