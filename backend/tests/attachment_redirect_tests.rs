mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::core::models::{Account, Attachment, Message};
use backend::core::repository::{AccountRepository as _, MessageRepository as _};
use backend::core::types::DbUuid;
use backend::db::pool::DbPool;
use backend::db::sqlite::account_repository::SqliteAccountRepository;
use backend::db::sqlite::attachment_repository::AttachmentRepository;
use backend::db::sqlite::message_repository::SqliteMessageRepository;
use backend::plugins::manager::PluginManager;
use backend::plugins::mock::MockProviderPlugin;
use common::setup_test_db;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tower::ServiceExt;
use uuid::Uuid;

/// Helper to create a test AppState with in-memory SQLite and a registered MockProviderPlugin.
async fn create_test_state_with_mock() -> AppState {
    let db = setup_test_db().await;
    let jwt_secret = "test_jwt_secret_12345".to_string();
    let mut manager = PluginManager::new();
    manager.register(Box::new(MockProviderPlugin::new("gmail", "Gmail")));
    let plugin_manager = Arc::new(RwLock::new(manager));
    let (sync_tx, _) = broadcast::channel(100);

    AppState {
        db,
        jwt_secret,
        plugin_manager,
        sync_tx,
        auth_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(100, Duration::from_secs(60)),
    }
}

/// Helper to register a user and return (User UUID, JWT token)
async fn create_user_and_token(app: &axum::Router, username: &str) -> (Uuid, String) {
    let register_payload = json!({
        "username": username,
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
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let user_id = Uuid::parse_str(body_json.get("user_id").unwrap().as_str().unwrap()).unwrap();

    let token_payload = json!({
        "username": username,
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
    let token_bytes = axum::body::to_bytes(token_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let token_json: serde_json::Value = serde_json::from_slice(&token_bytes).unwrap();
    let token = token_json
        .get("token")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    (user_id, token)
}

#[tokio::test]
async fn test_attachment_redirect_happy_path() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_a@kestrel.dev").await;

    // Seed Account A for User A
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-123".to_string(),
        display_name: "User A Gmail".to_string(),
        access_token: Some("mock_access_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message A for Account A
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-1".to_string(),
        thread_id: "thread-1".to_string(),
        subject: Some("Test Email".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_a@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 1024,
            external_id: Some("ext-att-123".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    // Request GET /api/v1/messages/{id}/attachments/{filename}/redirect
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/test.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::FOUND {
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "Expected 302 FOUND, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    let location_header = response
        .headers()
        .get("location")
        .expect("Location header should be set")
        .to_str()
        .unwrap();
    assert_eq!(
        location_header,
        "https://cdn.kestrel.dev/attachments/msg-ext-1/ext-att-123"
    );
}

#[tokio::test]
async fn test_attachment_redirect_unauthenticated() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state);

    let message_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/test.pdf/redirect",
                    message_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_attachment_redirect_unowned_account() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_a_id, _token_a) = create_user_and_token(&app, "user_a_unowned@kestrel.dev").await;
    let (_user_b_id, token_b) = create_user_and_token(&app, "user_b_unowned@kestrel.dev").await;

    // Seed Account A owned by User A
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_a_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-123".to_string(),
        display_name: "User A Account".to_string(),
        access_token: Some("mock_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message A under Account A
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-1".to_string(),
        thread_id: "thread-1".to_string(),
        subject: Some("Test Email".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_a@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();
    }

    // User B tries to access User A's message attachment -> 403 FORBIDDEN
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/test.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token_b))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    if response.status() != StatusCode::FORBIDDEN {
        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        panic!(
            "Expected 403 FORBIDDEN, got {} with body: {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
}

#[tokio::test]
async fn test_attachment_redirect_nonexistent_message() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state);

    let (_user_id, token) = create_user_and_token(&app, "user_nonexistent_msg@kestrel.dev").await;
    let random_message_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/test.pdf/redirect",
                    random_message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_attachment_redirect_nonexistent_filename() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_nonexistent_file@kestrel.dev").await;

    // Seed Account A
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-123".to_string(),
        display_name: "User Account".to_string(),
        access_token: Some("mock_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message A
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-1".to_string(),
        thread_id: "thread-1".to_string(),
        subject: Some("Test Email".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        // Insert attachment with filename "real.pdf"
        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "real.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 1024,
            external_id: Some("ext-att-123".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    // Request non-existent filename "nonexistent.png"
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/nonexistent.png/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_attachment_redirect_legacy_alias() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_alias@kestrel.dev").await;

    // Seed Account
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-alias-123".to_string(),
        display_name: "User Alias Gmail".to_string(),
        access_token: Some("mock_access_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-alias".to_string(),
        thread_id: "thread-alias".to_string(),
        subject: Some("Alias Test Email".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_alias@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello Alias".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "alias_doc.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 2048,
            external_id: Some("ext-att-alias-999".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    // Request legacy route alias: GET /api/messages/{id}/attachments/{filename}/redirect
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/messages/{}/attachments/alias_doc.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FOUND);
    let location_header = response
        .headers()
        .get("location")
        .expect("Location header should be set")
        .to_str()
        .unwrap();
    assert_eq!(
        location_header,
        "https://cdn.kestrel.dev/attachments/msg-ext-alias/ext-att-alias-999"
    );
}

#[tokio::test]
async fn test_attachment_redirect_missing_external_id() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_no_ext_id@kestrel.dev").await;

    // Seed Account
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-noext".to_string(),
        display_name: "User Gmail".to_string(),
        access_token: Some("mock_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-noext".to_string(),
        thread_id: "thread-noext".to_string(),
        subject: Some("No Ext ID".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_no_ext_id@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        // Attachment without external_id (None)
        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "no_ext.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 512,
            external_id: None,
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/no_ext.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attachment_redirect_missing_access_token() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_no_token@kestrel.dev").await;

    // Seed Account with access_token: None
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-notoken".to_string(),
        display_name: "User Gmail No Token".to_string(),
        access_token: None,
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-notoken".to_string(),
        thread_id: "thread-notoken".to_string(),
        subject: Some("No Token".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_no_token@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "file.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 512,
            external_id: Some("ext-123".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/file.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attachment_redirect_missing_provider_plugin() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_no_plugin@kestrel.dev").await;

    // Seed Account with provider "unregistered_provider"
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "unregistered_provider".to_string(),
        provider_account_id: "unregistered-123".to_string(),
        display_name: "Unregistered Provider Account".to_string(),
        access_token: Some("mock_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-noplugin".to_string(),
        thread_id: "thread-noplugin".to_string(),
        subject: Some("No Plugin Test".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_no_plugin@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "noplugin.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 512,
            external_id: Some("ext-noplugin-123".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/noplugin.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_attachment_redirect_route_syntax_dual_registration() {
    let state = create_test_state_with_mock().await;
    let app = create_router(state.clone());

    let (user_id, token) = create_user_and_token(&app, "user_syntax_test@kestrel.dev").await;

    // Seed Account A
    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid(account_id),
        user_id: DbUuid(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google-syntax-123".to_string(),
        display_name: "Syntax Test Gmail".to_string(),
        access_token: Some("mock_access_token".to_string()),
        refresh_token: None,
        token_expires_at: None,
        created_at: 0,
        updated_at: 0,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteAccountRepository::new(pool.clone());
        repo.create(&account).await.unwrap();
    }

    // Seed Message A
    let message_id = Uuid::new_v4();
    let message = Message {
        id: DbUuid(message_id),
        account_id: DbUuid(account_id),
        external_id: "msg-ext-syntax".to_string(),
        thread_id: "thread-syntax".to_string(),
        subject: Some("Syntax Test Email".to_string()),
        sender_name: Some("Sender".to_string()),
        sender_email: "sender@example.com".to_string(),
        recipients: "user_syntax_test@kestrel.dev".to_string(),
        date_sent: 1000,
        date_received: 1000,
        snippet: Some("Hello Syntax".to_string()),
        body_text: None,
        body_html: None,
        labels: None,
        is_read: false,
        is_archived: false,
        is_deleted: false,
        has_attachments: true,
        snoozed_until: None,
        created_at: 1000,
        updated_at: 1000,
    };

    if let DbPool::Sqlite(pool) = &state.db {
        let repo = SqliteMessageRepository::new(pool.clone());
        repo.upsert(&message).await.unwrap();

        let att_repo = AttachmentRepository::new(Arc::new(pool.clone()));
        let attachment = Attachment {
            id: DbUuid(Uuid::new_v4()),
            message_id: DbUuid(message_id),
            filename: "syntax_test.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            size: 1024,
            external_id: Some("ext-att-syntax-123".to_string()),
            created_at: 1000,
        };
        att_repo.create_attachment(&attachment).await.unwrap();
    }

    // 1. Verify standard curly brace syntax route (/api/v1/messages/{id}/attachments/{filename}/redirect)
    let res_v1_curly = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/messages/{}/attachments/syntax_test.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_v1_curly.status(), StatusCode::FOUND);
    assert_eq!(
        res_v1_curly
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "https://cdn.kestrel.dev/attachments/msg-ext-syntax/ext-att-syntax-123"
    );

    // 2. Verify legacy alias curly brace syntax route (/api/messages/{id}/attachments/{filename}/redirect)
    let res_alias_curly = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/messages/{}/attachments/syntax_test.pdf/redirect",
                    message_id
                ))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_alias_curly.status(), StatusCode::FOUND);
    assert_eq!(
        res_alias_curly
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
        "https://cdn.kestrel.dev/attachments/msg-ext-syntax/ext-att-syntax-123"
    );

    // 3. Verify colon syntax route registration (/api/v1/messages/:id/attachments/:filename/redirect)
    let res_v1_colon = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/messages/:id/attachments/:filename/redirect")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Reaches handler extractor (returns 400 Bad Request because ":id" is not a valid UUID), not 404
    assert_ne!(res_v1_colon.status(), StatusCode::NOT_FOUND);

    // 4. Verify legacy alias colon syntax route registration (/api/messages/:id/attachments/:filename/redirect)
    let res_alias_colon = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/messages/:id/attachments/:filename/redirect")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Reaches handler extractor (returns 400 Bad Request because ":id" is not a valid UUID), not 404
    assert_ne!(res_alias_colon.status(), StatusCode::NOT_FOUND);
}
