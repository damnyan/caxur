use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::roles::update::{UpdateRoleRequest, UpdateRoleUseCase};
use caxur::domain::roles::RoleRepository;
use caxur::infrastructure::repositories::roles::PostgresRoleRepository;
use caxur::shared::error::AppError;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_update_role_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = UpdateRoleUseCase::new(repo.clone());

    // Create a role
    let prefix = Uuid::new_v4().to_string();
    let new_role = caxur::domain::roles::NewRole {
        name: format!("role_{}", prefix),
        description: Some("Original description".to_string()),
    };
    let role = repo.create(new_role).await.expect("Failed to create role");

    // Update the role
    let req = UpdateRoleRequest {
        name: Some(format!("updated_role_{}", prefix)),
        description: Some("Updated description".to_string()),
    };

    let updated_role = use_case
        .execute(role.id, req)
        .await
        .expect("Failed to update role");

    assert_eq!(updated_role.name, format!("updated_role_{}", prefix));
    assert_eq!(
        updated_role.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
#[serial]
async fn test_update_role_not_found() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = UpdateRoleUseCase::new(repo);

    let req = UpdateRoleRequest {
        name: Some("irrelevant".to_string()),
        description: None,
    };

    let result = use_case.execute(Uuid::new_v4(), req).await;

    match result {
        Err(AppError::NotFound(msg)) => {
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

#[tokio::test]
#[serial]
async fn test_update_role_name_conflict() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = UpdateRoleUseCase::new(repo.clone());

    let prefix = Uuid::new_v4().to_string();
    // Role 1
    let role1 = repo
        .create(caxur::domain::roles::NewRole {
            name: format!("role1_{}", prefix),
            description: None,
        })
        .await
        .unwrap();

    // Role 2
    let role2 = repo
        .create(caxur::domain::roles::NewRole {
            name: format!("role2_{}", prefix),
            description: None,
        })
        .await
        .unwrap();

    // Update Role 2 to name of Role 1
    let req = UpdateRoleRequest {
        name: Some(role1.name),
        description: None,
    };

    let result = use_case.execute(role2.id, req).await;

    match result {
        Err(AppError::ValidationError(msg)) => {
            assert_eq!(msg, "Role name already exists");
        }
        _ => panic!(
            "Expected ValidationError(Role name already exists), got {:?}",
            result
        ),
    }
}
