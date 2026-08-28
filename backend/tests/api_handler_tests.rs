mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::core::models::SettingsPayload;
use backend::plugins::manager::PluginManager;
use common::{TEST_SECRET, seed_account, seed_calendar, seed_contact, seed_message, setup_test_db};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tower::ServiceExt;

/// Helper to create a test AppState with in-memory SQLite
async fn create_test_state() -> AppState {
    let db = setup_test_db().await;
    let jwt_secret = TEST_SECRET.to_string();
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new()));
    let (sync_tx, _) = broadcast::channel(100);
    let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);

    AppState {
        db,
        jwt_secret,
        plugin_manager,
        sync_tx,
        sync_job_tx,
        auth_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
    }
}

/// Register a test user and return (user_id_str, token_str)
async fn register_and_get_token(app: &axum::Router, username: &str) -> (String, String) {
    let register_payload = json!({
        "username": username,
        "password": "Password123!"
    });

    let res = app
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
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user_id = reg_json["user_id"].as_str().unwrap().to_string();

    let token_payload = json!({
        "username": username,
        "password": "Password123!"
    });

    let token_res = app
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
    assert_eq!(token_res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(token_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let token_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = token_json["token"].as_str().unwrap().to_string();

    (user_id, token)
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// === Auth & User Lifecycle Tests ===

#[tokio::test]
async fn test_auth_me_endpoint() {
    let state = create_test_state().await;
    let app = create_router(state);
    let (user_id, token) = register_and_get_token(&app, "me_user@kestrel.dev").await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["user_id"], user_id);
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["providers"].as_array().is_some());
}

// === Message Handler Tests ===

#[tokio::test]
async fn test_messages_crud_and_actions() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register_and_get_token(&app, "msg_handler@kestrel.dev").await;
    let user_uuid = uuid::Uuid::parse_str(&user_id).unwrap();

    // Seed account & messages directly in state.db
    let account = seed_account(&state.db, user_uuid, "gmail", "Work Gmail").await;
    let msg1 = seed_message(
        &state.db,
        account.id.0,
        "Quarterly Roadmap Review",
        "lead@company.com",
        "Please find the attached Q3 roadmap.",
        Some("inbox"),
        false,
    )
    .await;
    let msg2 = seed_message(
        &state.db,
        account.id.0,
        "Weekly Newsletter",
        "news@tech.io",
        "Top engineering stories of the week.",
        Some("inbox"),
        true,
    )
    .await;

    // 1. List messages
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/messages")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let messages = json["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(json["total"], 2);

    // 2. Get message detail
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/messages/{}", msg1.id.0))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let msg_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(msg_json["subject"], "Quarterly Roadmap Review");
    assert_eq!(msg_json["sender_email"], "lead@company.com");
    assert_eq!(msg_json["is_read"], false);

    // 3. Mark as read
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/read", msg1.id.0))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(json!({"read": true}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 4. Toggle star
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/star", msg1.id.0))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(json!({"is_starred": true}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 5. Update labels
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/messages/{}/labels", msg1.id.0))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"labels": ["inbox", "important", "work"]}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 6. Bulk action (archive both messages)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/messages/bulk")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "message_ids": [msg1.id.0, msg2.id.0],
                        "action": "archive"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

// === Calendar & Event Handler Tests ===

#[tokio::test]
async fn test_calendar_and_event_full_lifecycle() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register_and_get_token(&app, "cal_handler@kestrel.dev").await;
    let user_uuid = uuid::Uuid::parse_str(&user_id).unwrap();

    // Seed account & calendar
    let account = seed_account(&state.db, user_uuid, "google", "Google Calendar").await;
    let cal = seed_calendar(&state.db, account.id.0, "Engineering", true).await;

    // 1. List calendars
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/calendars")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let cals = json["calendars"].as_array().unwrap();
    assert_eq!(cals.len(), 1);
    assert_eq!(cals[0]["name"], "Engineering");

    // 2. Create event in valid calendar
    let now = chrono::Utc::now().timestamp();
    let create_payload = json!({
        "calendar_id": cal.id.0,
        "title": "Architecture Deep-Dive",
        "description": "Discuss Rust Axum & Specta architecture",
        "location": "Conf Room 3B",
        "start_time": now + 3600,
        "end_time": now + 7200,
        "is_all_day": false,
        "attendees": r#"[{"name": "Lead", "email": "lead@kestrel.dev", "status": "accepted"}]"#
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/events")
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let ev_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let event_id = ev_json["id"].as_str().unwrap().to_string();
    assert!(!event_id.is_empty());

    // 3. Query events with range parameters
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/events?start_time={}&end_time={}",
                    now,
                    now + 10000
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let list_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let events = list_json["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["title"], "Architecture Deep-Dive");

    // 4. Update the event
    let update_payload = json!({
        "calendar_id": cal.id.0,
        "title": "Architecture Deep-Dive (Rescheduled)",
        "start_time": now + 4000,
        "end_time": now + 7600
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/v1/events/{}", event_id))
                .header("authorization", format!("Bearer {}", token))
                .header("content-type", "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 5. Delete the event
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/events/{}", event_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}

// === Contacts Search Handler Test ===

#[tokio::test]
async fn test_contacts_search_endpoint() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register_and_get_token(&app, "contact_api@kestrel.dev").await;
    let user_uuid = uuid::Uuid::parse_str(&user_id).unwrap();

    let account = seed_account(&state.db, user_uuid, "gmail", "Personal").await;
    seed_contact(
        &state.db,
        account.id.0,
        "Sarah Connor",
        "sarah@resistance.org",
    )
    .await;
    seed_contact(
        &state.db,
        account.id.0,
        "John Connor",
        "john@resistance.org",
    )
    .await;

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/contacts/search?q=Sarah")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let contacts = json.as_array().unwrap();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0]["name"], "Sarah Connor");
}

// === Settings Handler Tests ===

#[tokio::test]
async fn test_get_and_put_settings_full_cycle() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (_, token) = register_and_get_token(&app, "settings_user@kestrel.dev").await;

    // 1. Initial GET /api/settings should be default
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/settings")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings_resp: SettingsPayload = serde_json::from_slice(&body).unwrap();
    assert_eq!(settings_resp, SettingsPayload::default());

    // 2. PUT /api/settings with partial updates
    let res = app
        .clone()
        .oneshot(
            Request::builder()
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
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. GET /api/settings again to verify persistence
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/settings")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let settings_resp: SettingsPayload = serde_json::from_slice(&body).unwrap();
    assert_eq!(settings_resp.mail_dense_mode, Some(true));
    assert_eq!(settings_resp.sync_interval, Some(600));
}

// === Security & Multi-Tenant Isolation Tests ===

#[tokio::test]
async fn test_unauthenticated_requests_rejected() {
    let state = create_test_state().await;
    let app = create_router(state);

    let endpoints = vec![
        ("/api/v1/messages", "GET"),
        ("/api/v1/calendars", "GET"),
        ("/api/v1/events", "GET"),
        ("/api/settings", "GET"),
        ("/api/v1/accounts", "GET"),
        ("/api/contacts/search?q=test", "GET"),
        ("/api/v1/search?q=test", "GET"),
    ];

    for (uri, method) in endpoints {
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            res.status(),
            StatusCode::UNAUTHORIZED,
            "Endpoint {} should require authentication",
            uri
        );
    }
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let state = create_test_state().await;
    let app = create_router(state.clone());

    let (user_a, token_a) = register_and_get_token(&app, "user_a@kestrel.dev").await;
    let (_user_b, token_b) = register_and_get_token(&app, "user_b@kestrel.dev").await;

    let user_a_uuid = uuid::Uuid::parse_str(&user_a).unwrap();
    let account_a = seed_account(&state.db, user_a_uuid, "gmail", "User A Account").await;
    let msg_a = seed_message(
        &state.db,
        account_a.id.0,
        "Private User A Secret",
        "user_a@kestrel.dev",
        "Secret content for user A only",
        None,
        false,
    )
    .await;

    // User A can read own message
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/messages/{}", msg_a.id.0))
                .header("authorization", format!("Bearer {}", token_a))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // User B cannot read User A's message (returns 404)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/messages/{}", msg_a.id.0))
                .header("authorization", format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_webhooks_ingestion() {
    let state = create_test_state().await;
    let app = create_router(state);

    // 1. Google PubSub webhook
    let google_payload = json!({
        "message": {
            "data": "eyJlbWFpbEFkZHJlc3MiOiJkZW1vQGxvY2FsIiwiaGlzdG9yeUlkIjoiMTIzNCJ9",
            "messageId": "msg-001"
        }
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/webhooks/google")
                .header("content-type", "application/json")
                .body(Body::from(google_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Microsoft Graph validation token handshake
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/webhooks/microsoft?validationToken=testValidationToken999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
