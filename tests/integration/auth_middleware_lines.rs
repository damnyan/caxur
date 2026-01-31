use crate::common;
use axum::{
    Extension, Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use caxur::domain::auth::AuthService;
use caxur::domain::permissions::Permission;
use caxur::presentation::middleware::auth::{RequiredPermissions, check_permissions};
use serial_test::serial;
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn test_cover_line_35_user_type_mismatch() {
    // Target Line 35: "Access denied: User {} type '{}' does not match required '{}'"
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = uuid::Uuid::new_v4();
    let auth_service = common::create_test_auth_service();
    // User has type "user"
    let token = auth_service
        .generate_access_token(user_id, "user".to_string())
        .expect("Failed to generate token");

    let state = common::create_test_app_state(pool.clone());

    // Route requires "admin" type
    let required_permissions = RequiredPermissions {
        user_type: "admin",
        permissions: vec![],
    };

    let app = Router::new()
        .route("/protected", get(|| async { "Success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            check_permissions,
        ))
        .layer(Extension(required_permissions))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_cover_lines_47_48_db_error() {
    // Target Lines 47-48: tracing::error!("Failed to fetch permissions..."); AppError::InternalServerError(e)
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let (admin_id, token) = common::create_admin_with_permissions(&pool).await;
    let state = common::create_test_app_state(pool.clone());

    let required_permissions = RequiredPermissions {
        user_type: "admin",
        permissions: vec![Permission::Wildcard], // Requires DB check
    };

    let app = Router::new()
        .route("/protected", get(|| async { "Success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            check_permissions,
        ))
        .layer(Extension(required_permissions))
        .with_state(state);

    // Force DB error by closing pool
    pool.close().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // cleanup intentionally skipped or best-effort as pool is closed
}

#[tokio::test]
#[serial]
async fn test_cover_line_60_insufficient_perms() {
    // Target Line 60: "Access denied: User {} lacks required permissions {:?}"
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    // Create admin WITHOUT permissions (just basic admin record)
    let admin_id = uuid::Uuid::new_v4();
    let email = "noperms@example.com";
    sqlx::query!(
        "INSERT INTO user_administrators (id, email, password_hash, first_name, last_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
        admin_id,
        email,
        "hash",
        "No",
        "Perms"
    )
    .execute(&pool)
    .await
    .unwrap();

    let token = common::generate_admin_token(admin_id);
    let state = common::create_test_app_state(pool.clone());

    // Context requires Wildcard, but user has NONE
    let required_permissions = RequiredPermissions {
        user_type: "admin",
        permissions: vec![Permission::Wildcard],
    };

    let app = Router::new()
        .route("/protected", get(|| async { "Success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            check_permissions,
        ))
        .layer(Extension(required_permissions))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_cover_lines_69_70_rbac_not_impl() {
    // Target Lines 69-70: "RBAC not implemented for user type: {}"; Forbidden
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = uuid::Uuid::new_v4();
    let auth_service = common::create_test_auth_service();
    // User type "merchant"
    let token = auth_service
        .generate_access_token(user_id, "merchant".to_string())
        .expect("Failed to generate token");

    let state = common::create_test_app_state(pool.clone());

    // Route requires "merchant"
    // Since "merchant" != "admin", it goes to the else block (Line 66)
    let required_permissions = RequiredPermissions {
        user_type: "merchant",
        permissions: vec![],
    };

    let app = Router::new()
        .route("/protected", get(|| async { "Success" }))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            check_permissions,
        ))
        .layer(Extension(required_permissions))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/protected")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}
