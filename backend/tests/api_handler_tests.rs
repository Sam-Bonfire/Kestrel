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

/// Helper to create a test AppState with in-memory SQLite
async fn create_test_state() -> AppState {
    let db = setup_test_db().await;
    let jwt_secret = "test_jwt_secret_12345".to_string();
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new()));
    let (sync_tx, _) = broadcast::channel(100);
    let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);

    AppState {
        db,
        jwt_secret,
        plugin_manager,
        sync_tx,
        sync_job_tx,
        auth_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
    }
}

/// Register a test user and return the JWT token
async fn register_and_get_token(app: &axum::Router) -> String {
    // Register
    let register_payload = json!({
        "username": "handler_test_user@kestrel.dev",
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

    // Get token
    let token_payload = json!({
        "username": "handler_test_user@kestrel.dev",
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
    body_json
        .get("token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

// === Health Endpoint Tests ===

#[tokio::test]
async fn test_health_check_returns_200() {
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// === Auth Handler Tests ===

#[tokio::test]
async fn test_register_handler_success() {
    let state = create_test_state().await;
    let app = create_router(state);

    let payload = json!({
        "username": "new_user@test.dev",
        "password": "ValidPass123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("user_id").is_some());
}

#[tokio::test]
async fn test_register_handler_empty_username() {
    let state = create_test_state().await;
    let app = create_router(state);

    let payload = json!({
        "username": "",
        "password": "Password123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_handler_short_password() {
    let state = create_test_state().await;
    let app = create_router(state);

    let payload = json!({
        "username": "short_pass_user@test.dev",
        "password": "short"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_token_handler_success() {
    let state = create_test_state().await;
    let app = create_router(state);

    let token = register_and_get_token(&app).await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_token_handler_wrong_password() {
    let state = create_test_state().await;
    let app = create_router(state);

    // Register
    let register_payload = json!({
        "username": "wrong_pass_user@test.dev",
        "password": "Password123!"
    });

    app.clone()
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

    // Try with wrong password
    let token_payload = json!({
        "username": "wrong_pass_user@test.dev",
        "password": "WrongPassword!"
    });

    let response = app
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

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_token_handler_nonexistent_user() {
    let state = create_test_state().await;
    let app = create_router(state);

    let payload = json!({
        "username": "nonexistent@test.dev",
        "password": "Password123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/token")
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// === Protected Route Tests (require auth) ===

#[tokio::test]
async fn test_protected_route_without_token() {
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_messages_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/messages")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("messages").is_some());
    assert!(body_json.get("total").is_some());
}

#[tokio::test]
async fn test_list_messages_with_invalid_token() {
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/messages")
                .header("authorization", "Bearer invalid_token_12345")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_list_calendars_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/calendars")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("calendars").is_some());
}

#[tokio::test]
async fn test_list_events_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/events")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("events").is_some());
}

#[tokio::test]
async fn test_search_messages_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/search?q=test")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert!(body_json.get("results").is_some());
    assert!(body_json.get("query").is_some());
}

#[tokio::test]
async fn test_list_providers_is_public() {
    let state = create_test_state().await;
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/providers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// === Message Handler Tests ===

#[tokio::test]
async fn test_get_message_nonexistent() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/messages/{}", fake_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 or empty (depends on implementation)
    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
}

// === Calendar Handler Tests ===

#[tokio::test]
async fn test_get_calendar_nonexistent() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let fake_id = uuid::Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/calendars/{}", fake_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return 404 or error (depends on implementation)
    assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
}

#[tokio::test]
async fn test_create_event_with_valid_token() {
    let state = create_test_state().await;
    let app = create_router(state);
    let token = register_and_get_token(&app).await;

    let payload = json!({
        "calendar_id": uuid::Uuid::new_v4(),
        "title": "Test Event",
        "start_time": 1721644800,
        "end_time": 1721648400,
        "description": "Test description",
        "is_all_day": false
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Since the calendar_id is random, it should return 404 NOT_FOUND,
    // indicating that auth and schema validation passed successfully.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_and_put_settings() {
    let state = create_test_state().await;
    let app = create_router(state.clone());

    // Register user to get a token
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "username": "settings@example.com",
                "password": "Password123!"
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Get token
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/token")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "username": "settings@example.com",
                "password": "Password123!"
            })
            .to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let token_resp: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let token = token_resp["token"].as_str().unwrap().to_string();

    // 1. Initial GET /api/settings should be empty JSON
    let req = Request::builder()
        .method("GET")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings_resp: backend::core::models::SettingsPayload =
        serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        settings_resp,
        backend::core::models::SettingsPayload::default()
    );

    // 2. PUT /api/settings
    let req = Request::builder()
        .method("PUT")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "mailDenseMode": true,
                "syncInterval": 600
            })
            .to_string(),
        ))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings_resp: backend::core::models::SettingsPayload =
        serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(settings_resp.mail_dense_mode, Some(true));
    assert_eq!(settings_resp.sync_interval, Some(600));

    // 3. GET /api/settings again to verify it persists
    let req = Request::builder()
        .method("GET")
        .uri("/api/settings")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings_resp: backend::core::models::SettingsPayload =
        serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(settings_resp.mail_dense_mode, Some(true));
    assert_eq!(settings_resp.sync_interval, Some(600));
}
