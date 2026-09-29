mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::plugins::manager::PluginManager;
use common::{seed_account, seed_message, setup_test_db};
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
async fn set_aside_roundtrip() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "aside1").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let msg = seed_message(
        &state.db,
        account.id.0,
        "Subject",
        "a@b.com",
        "body text",
        None,
        true,
    )
    .await;

    let (status, _) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/messages/{}/set-aside", msg.id.0),
        Some(json!({"is_set_aside": true})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, detail) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/messages/{}", msg.id.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["is_set_aside"], true);

    let (status, _) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/messages/{}/set-aside", msg.id.0),
        Some(json!({"is_set_aside": false})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, detail) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/messages/{}", msg.id.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["is_set_aside"], false);
}

#[tokio::test]
async fn set_aside_rejects_strangers_and_ghosts() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "aside2a").await;
    let (_, other_token) = register(&app, "aside2b").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let msg = seed_message(
        &state.db,
        account.id.0,
        "Subject",
        "a@b.com",
        "body text",
        None,
        true,
    )
    .await;

    let (status, _) = authed(
        &app,
        &other_token,
        "POST",
        &format!("/api/v1/messages/{}/set-aside", msg.id.0),
        Some(json!({"is_set_aside": true})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/messages/{}/set-aside", Uuid::new_v4()),
        Some(json!({"is_set_aside": true})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn sync_reupsert_preserves_set_aside() {
    use backend::core::repository::MessageRepository;
    use backend::db::sqlite::message_repository::SqliteMessageRepository;
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "aside3").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let msg = seed_message(
        &state.db,
        account.id.0,
        "Subject",
        "a@b.com",
        "body text",
        None,
        true,
    )
    .await;

    let (status, _) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/messages/{}/set-aside", msg.id.0),
        Some(json!({"is_set_aside": true})),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Simulate a provider re-sync touching the same row: round-trip the
    // struct the way sync does and upsert with an updated subject.
    let pool = common::get_sqlite_pool(&state.db);
    let repo = SqliteMessageRepository::new(pool);
    let mut round_tripped = repo.find_by_id(msg.id.0).await.unwrap().unwrap();
    assert!(round_tripped.is_set_aside);
    round_tripped.subject = Some("Updated subject".to_string());
    repo.upsert(&round_tripped).await.unwrap();

    let after = repo.find_by_id(msg.id.0).await.unwrap().unwrap();
    assert!(after.is_set_aside, "re-sync must preserve the flag");
    assert_eq!(after.subject.as_deref(), Some("Updated subject"));
}
