use crate::common;
use crate::setup_test_db_or_skip;
use caxur::application::roles::create::{CreateRoleRequest, CreateRoleUseCase};
use caxur::infrastructure::repositories::roles::PostgresRoleRepository;
use serial_test::serial;
use std::sync::Arc;

#[tokio::test]
#[serial]
async fn test_create_role_success() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = CreateRoleUseCase::new(repo);

    let prefix = uuid::Uuid::new_v4().to_string();
    let req = CreateRoleRequest {
        name: format!("Role_{}", prefix),
        description: Some("Test description".to_string()),
        scope: "ADMINISTRATOR".to_string(),
        group_id: None,
    };

    let role = use_case.execute(req).await.expect("Failed to create role");

    assert!(role.name.contains(&prefix));
    assert_eq!(role.description, Some("Test description".to_string()));
}

#[tokio::test]
#[serial]
async fn test_create_role_duplicate_name() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;
    let repo = Arc::new(PostgresRoleRepository::new(pool.clone()));
    let use_case = CreateRoleUseCase::new(repo);

    let prefix = uuid::Uuid::new_v4().to_string();
    let name = format!("DupRole_{}", prefix);

    let req1 = CreateRoleRequest {
        name: name.clone(),
        description: None,
        scope: "ADMINISTRATOR".to_string(),
        group_id: None,
    };
    use_case
        .execute(req1)
        .await
        .expect("Failed to create first role");

    let req2 = CreateRoleRequest {
        name,
        description: Some("Duplicate".to_string()),
        scope: "ADMINISTRATOR".to_string(),
        group_id: None,
    };
    let result = use_case.execute(req2).await;

    assert!(result.is_err());
}
