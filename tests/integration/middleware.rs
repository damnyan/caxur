use crate::common;

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{HeaderValue, Request, StatusCode},
};
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
