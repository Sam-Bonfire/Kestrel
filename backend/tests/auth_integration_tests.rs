mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::plugins::manager::PluginManager;
use common::setup_test_db;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tower::ServiceExt;

#[tokio::test]
async fn test_auth_registration_and_token_flow() {
    let db = setup_test_db().await;
    let jwt_secret = "super_secret_jwt_key_for_testing_12345".to_string();
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new()));
    let (sync_tx, _) = broadcast::channel(100);

    let state = AppState {
        db,
        jwt_secret,
        plugin_manager,
        sync_tx,
        auth_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
    };

    let app = create_router(state);

    // 1. Register a new user
    let register_payload = json!({
        "username": "auth_test_user@kestrel.dev",
        "password": "Password123!"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Request a JWT token for the user
    let token_payload = json!({
        "username": "auth_test_user@kestrel.dev",
        "password": "Password123!"
    });

    let token_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/token")
                .header("content-type", "application/json")
                .body(Body::from(token_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(token_response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(token_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(body_json.get("token").is_some());
    let token_str = body_json.get("token").unwrap().as_str().unwrap();
    assert!(!token_str.is_empty());

    // 3. Access protected route with Bearer token
    let protected_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/messages")
                .header("authorization", format!("Bearer {}", token_str))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(protected_response.status(), StatusCode::OK);
}
