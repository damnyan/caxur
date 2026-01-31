use crate::common;

use axum::{
    Extension, Router,
    body::Body,
    extract::ConnectInfo,
    http::{HeaderValue, Request, StatusCode},
    middleware,
    routing::get,
};
use caxur::domain::auth::AuthService;
use caxur::domain::permissions::Permission;
use caxur::presentation::middleware::auth::{RequiredPermissions, check_permissions};
use serial_test::serial;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt;

#[tokio::test]
#[serial]
async fn test_cors_middleware() {
    // Setup env for CORS
    unsafe {
        std::env::set_var("CORS_ALLOWED_ORIGINS", "http://test.com");
    }

    let pool = match common::setup_test_db().await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test_cors_middleware: database not available");
            return;
        }
    };

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Test Preflight
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("Origin", "http://test.com")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin"),
        Some(&HeaderValue::from_static("http://test.com"))
    );

    common::cleanup_test_db(&pool).await;
    unsafe {
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}

#[tokio::test]
#[serial]
async fn test_rate_limit_middleware() {
    // Setup env for Rate Limit (2 per minute)
    unsafe {
        std::env::set_var("RATE_LIMIT_PER_MINUTE", "2");
    }

    let pool = match common::setup_test_db().await {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Skipping test_rate_limit_middleware: database not available");
            return;
        }
    };

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);

    // Request 1 - OK
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .extension(ConnectInfo(addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Request 2 - OK
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .extension(ConnectInfo(addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Request 3 - Too Many Requests
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .extension(ConnectInfo(addr))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    common::cleanup_test_db(&pool).await;
    unsafe {
        std::env::remove_var("RATE_LIMIT_PER_MINUTE");
    }
}

#[tokio::test]
#[serial]
async fn test_cors_middleware_wildcard() {
    // Setup env for CORS
    unsafe {
        std::env::set_var("CORS_ALLOWED_ORIGINS", "*");
    }

    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Test Preflight
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("Origin", "http://any-origin.com")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // When allowing Any, the response usually mirrors the origin or is *
    let origin = response
        .headers()
        .get("access-control-allow-origin")
        .unwrap();
    assert_eq!(origin, "*");

    common::cleanup_test_db(&pool).await;
    unsafe {
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}

#[tokio::test]
#[serial]
async fn test_cors_middleware_default() {
    // Setup env for CORS - Empty
    unsafe {
        std::env::set_var("CORS_ALLOWED_ORIGINS", "");
    }

    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let state = common::create_test_app_state(pool.clone());
    let app = caxur::presentation::router::app(state).unwrap();

    // Test Preflight
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("Origin", "http://another-origin.com")
                .header("Access-Control-Request-Method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let origin = response
        .headers()
        .get("access-control-allow-origin")
        .unwrap();
    assert_eq!(origin, "*");

    common::cleanup_test_db(&pool).await;
    unsafe {
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }
}

#[tokio::test]
#[serial]
async fn test_auth_middleware_admin_access() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let (_admin_id, token) = common::create_admin_with_permissions(&pool).await;
    let state = common::create_test_app_state(pool.clone());

    // Define a route that requires admin permission
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

    assert_eq!(response.status(), StatusCode::OK);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_middleware_insufficient_permissions() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    // Create admin without assigning role (no permissions)
    let admin_id = uuid::Uuid::new_v4();
    let email = "admin_noperm@example.com";
    let password_hash = "hash";

    sqlx::query!(
        "INSERT INTO user_administrators (id, email, password_hash, first_name, last_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
        admin_id,
        email,
        password_hash,
        "Admin",
        "NoPerm"
    )
    .execute(&pool)
    .await
    .unwrap();

    let token = common::generate_admin_token(admin_id);
    let state = common::create_test_app_state(pool.clone());

    // Route requires Wildcard, but user has none.
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
async fn test_auth_middleware_wrong_user_type() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = uuid::Uuid::new_v4();
    // Generate USER token, but endpoint requires ADMIN
    let token = common::generate_test_token(user_id);

    let state = common::create_test_app_state(pool.clone());

    // Even with empty permissions required, user type mismatch should fail if enforcing user_type
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
async fn test_auth_middleware_rbac_not_implemented() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let user_id = uuid::Uuid::new_v4();
    let auth_service = common::create_test_auth_service();
    // Generate token for "merchant" type
    let token = auth_service
        .generate_access_token(user_id, "merchant".to_string())
        .expect("Failed to generate token");

    let state = common::create_test_app_state(pool.clone());

    // Route requires "merchant" type, but RBAC logic for merchant is not implemented in middleware
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

    // Should return 403 Forbidden because "RBAC not implemented"
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_auth_middleware_db_error() {
    let pool = setup_test_db_or_skip!();
    common::cleanup_test_db(&pool).await;

    let (_admin_id, token) = common::create_admin_with_permissions(&pool).await;
    let state = common::create_test_app_state(pool.clone());

    // Route requires "admin" type, so it will try to fetch permissions from DB
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

    // CLOSE the pool to trigger DB error during permission fetch
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

    // Should return 500 Internal Server Error
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Note: cleanup_test_db might fail because pool is closed, but it's the end of test.
    // However, if we reuse the pool from a global setup (which we don't, we create new one), it's fine.
    // setup_test_db creates a NEW pool each time.
}
