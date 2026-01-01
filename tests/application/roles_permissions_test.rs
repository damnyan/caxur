use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::roles::attach_permission::AttachPermissionUseCase;
use caxur::application::roles::detach_permission::DetachPermissionUseCase;
use caxur::domain::permissions::Permission;
use caxur::domain::roles::RoleRepository;
use caxur::infrastructure::repositories::roles::PostgresRoleRepository;
use caxur::shared::error::AppError;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_attach_permission_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = AttachPermissionUseCase::new(repo.clone());

    let prefix = Uuid::new_v4().to_string();
    let role = repo
        .create(caxur::domain::roles::NewRole {
            name: format!("role_{}", prefix),
            description: None,
        })
        .await
        .expect("Failed to create role");

    let permissions = vec![
        Permission::AdministratorManagement,
        Permission::RoleManagement,
    ];

    use_case
        .execute(role.id, permissions.clone())
        .await
        .expect("Failed to attach permissions");

    let attached = repo
        .get_permissions(role.id)
        .await
        .expect("Failed to get permissions");
    // We expect attached permissions to contain what we added.
    // Note: implementation of get_permissions might differ, let's assume it returns Vec<Permission>.
    // Verify at least one match.
    assert!(attached.contains(&Permission::AdministratorManagement));
    assert!(attached.contains(&Permission::RoleManagement));
}

#[tokio::test]
#[serial]
async fn test_attach_permission_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = AttachPermissionUseCase::new(repo);

    let permissions = vec![Permission::AdministratorManagement];
    let result = use_case.execute(Uuid::new_v4(), permissions).await;

    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

#[tokio::test]
#[serial]
async fn test_detach_permission_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let attach_use_case = AttachPermissionUseCase::new(repo.clone());
    let detach_use_case = DetachPermissionUseCase::new(repo.clone());

    let prefix = Uuid::new_v4().to_string();
    let role = repo
        .create(caxur::domain::roles::NewRole {
            name: format!("role_d_{}", prefix),
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

    // Detach RoleManagement
    detach_use_case
        .execute(role.id, vec![Permission::RoleManagement])
        .await
        .expect("Failed to detach permission");

    let attached = repo
        .get_permissions(role.id)
        .await
        .expect("Failed to get permissions");
    assert!(attached.contains(&Permission::AdministratorManagement));
    assert!(!attached.contains(&Permission::RoleManagement));
}

#[tokio::test]
#[serial]
async fn test_detach_permission_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = DetachPermissionUseCase::new(repo);

    let permissions = vec![Permission::AdministratorManagement];
    let result = use_case.execute(Uuid::new_v4(), permissions).await;

    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}
