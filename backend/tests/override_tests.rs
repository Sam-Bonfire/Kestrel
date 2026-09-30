mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::plugins::manager::PluginManager;
use common::{seed_account, setup_test_db};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_state() -> AppState {
    let db = setup_test_db().await;
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new()));
    let (sync_tx, _) = broadcast::channel(100);
    let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);
    AppState {
        db,
        jwt_secret: common::TEST_SECRET.to_string(),
        plugin_manager,
        sync_tx,
        sync_job_tx,
        auth_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
    }
}

async fn register(app: &axum::Router, username: &str) -> (Uuid, String) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username": username, "password": "Password123!"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg: Value = serde_json::from_slice(&body).unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/token")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username": username, "password": "Password123!"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tok: Value = serde_json::from_slice(&body).unwrap();
    (
        reg["user_id"].as_str().unwrap().parse().unwrap(),
        tok["token"].as_str().unwrap().to_string(),
    )
}

async fn authed(
    app: &axum::Router,
    token: &str,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    let req_body = Body::from(body.map(|b| b.to_string()).unwrap_or_default());
    let res = app
        .clone()
        .oneshot(builder.body(req_body).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

#[tokio::test]
async fn subject_override_roundtrip() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "ovr1").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;

    let (status, _) = authed(
        &app,
        &token,
        "PUT",
        "/api/v1/threads/th-1/subject",
        Some(json!({"account_id": account.id.0, "subject": "  My rename  "})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, rows) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/threads/overrides?account_id={}", account.id.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["subject"], "My rename");
    assert_eq!(rows[0]["thread_id"], "th-1");

    // Overwrite wins.
    let (status, _) = authed(
        &app,
        &token,
        "PUT",
        "/api/v1/threads/th-1/subject",
        Some(json!({"account_id": account.id.0, "subject": "Second"})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Clear restores provider subject (row gone).
    let (status, _) = authed(
        &app,
        &token,
        "DELETE",
        &format!("/api/v1/threads/th-1/subject?account_id={}", account.id.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, rows) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/threads/overrides?account_id={}", account.id.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rows.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn subject_override_validation_and_ownership() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "ovr2a").await;
    let (_, other_token) = register(&app, "ovr2b").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;

    for body in [
        json!({"account_id": account.id.0, "subject": ""}),
        json!({"account_id": account.id.0, "subject": "x".repeat(201)}),
    ] {
        let (status, _) = authed(
            &app,
            &token,
            "PUT",
            "/api/v1/threads/th-9/subject",
            Some(body),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    // Stranger's account id -> 404, never leaks existence.
    let (status, _) = authed(
        &app,
        &other_token,
        "PUT",
        "/api/v1/threads/th-9/subject",
        Some(json!({"account_id": account.id.0, "subject": "Squat"})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Missing account_id on list/clear.
    let (status, _) = authed(&app, &token, "GET", "/api/v1/threads/overrides", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
