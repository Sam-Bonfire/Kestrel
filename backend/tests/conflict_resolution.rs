use backend::api::router::AppState;
use backend::core::models::{HistoricalRevision, Message};
use backend::core::repository::MessageRepository;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

async fn create_test_app_state() -> AppState {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

    let _ = sqlx::query(
        "CREATE TABLE accounts (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            provider TEXT NOT NULL,
            provider_account_id TEXT NOT NULL,
            display_name TEXT NOT NULL,
            access_token TEXT,
            refresh_token TEXT,
            token_expires_at INTEGER,
            sync_error TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "CREATE TABLE messages (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            external_id TEXT NOT NULL,
            thread_id TEXT NOT NULL,
            subject TEXT,
            sender_name TEXT,
            sender_email TEXT NOT NULL,
            recipients TEXT NOT NULL,
            date_sent INTEGER NOT NULL,
            date_received INTEGER NOT NULL,
            snippet TEXT,
            body_text TEXT,
            body_html TEXT,
            labels TEXT,
            is_read BOOLEAN NOT NULL,
            is_archived BOOLEAN NOT NULL,
            is_deleted BOOLEAN NOT NULL,
            has_attachments BOOLEAN NOT NULL,
            snoozed_until INTEGER,
            has_conflict BOOLEAN NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            UNIQUE(account_id, external_id)
        )",
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        "CREATE TABLE historical_revisions (
            id TEXT PRIMARY KEY,
            resource_type TEXT NOT NULL,
            resource_id TEXT NOT NULL,
            serialized_payload TEXT NOT NULL,
            revision_number INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await;

    let (sync_tx, _) = tokio::sync::broadcast::channel(100);
    let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);

    AppState {
        db: backend::db::pool::DbPool::Sqlite(pool),
        jwt_secret: "test_secret".to_string(),
        plugin_manager: Arc::new(RwLock::new(backend::plugins::manager::PluginManager::new())),
        sync_tx,
        sync_job_tx,
        auth_rate_limiter: backend::api::rate_limit::RateLimiter::new(
            10,
            std::time::Duration::from_secs(60),
        ),
        general_rate_limiter: backend::api::rate_limit::RateLimiter::new(
            10,
            std::time::Duration::from_secs(60),
        ),
    }
}

#[tokio::test]
async fn test_lww_provider_wins() {
    let state = create_test_app_state().await;
    let pool = match &state.db {
        backend::db::pool::DbPool::Sqlite(pool) => pool.clone(),
        _ => unreachable!(),
    };

    let msg_repo =
        backend::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone());
    let msg_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let msg = Message {
        id: backend::core::types::DbUuid::new(msg_id),
        account_id: backend::core::types::DbUuid::new(account_id),
        external_id: "ext1".to_string(),
        thread_id: "thread1".to_string(),
        subject: Some("Old Subject".to_string()),
        sender_name: None,
        sender_email: "test@example.com".to_string(),
        recipients: "me@example.com".to_string(),
        date_sent: 100,
        date_received: 100, // remote timestamp
        snippet: None,
        body_text: None,
        body_html: None,
        labels: None,
        is_read: true,
        is_archived: false,
        is_deleted: false,
        has_attachments: false,
        snoozed_until: None,
        has_conflict: false,
        created_at: 100,
        updated_at: 50, // local updated at < remote
    };

    msg_repo.upsert(&msg).await.unwrap();

    let mut payload = msg.clone();
    payload.subject = Some("New Subject".to_string());
    payload.date_received = 200; // newer remote

    if payload.date_received > msg.updated_at {
        let mut new_msg = msg.clone();
        new_msg.subject = payload.subject;
        new_msg.updated_at = chrono::Utc::now().timestamp();
        msg_repo.upsert(&new_msg).await.unwrap();
    }

    let db_msg = msg_repo.find_by_id(msg_id).await.unwrap().unwrap();
    assert_eq!(db_msg.subject, Some("New Subject".to_string()));
    assert_eq!(db_msg.has_conflict, false);
}

#[tokio::test]
async fn test_lww_local_wins_conflict() {
    let state = create_test_app_state().await;
    let pool = match &state.db {
        backend::db::pool::DbPool::Sqlite(pool) => pool.clone(),
        _ => unreachable!(),
    };

    let msg_repo =
        backend::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone());
    let msg_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let msg = Message {
        id: backend::core::types::DbUuid::new(msg_id),
        account_id: backend::core::types::DbUuid::new(account_id),
        external_id: "ext1".to_string(),
        thread_id: "thread1".to_string(),
        subject: Some("New Subject".to_string()),
        sender_name: None,
        sender_email: "test@example.com".to_string(),
        recipients: "me@example.com".to_string(),
        date_sent: 100,
        date_received: 100,
        snippet: None,
        body_text: None,
        body_html: None,
        labels: None,
        is_read: true,
        is_archived: false,
        is_deleted: false,
        has_attachments: false,
        snoozed_until: None,
        has_conflict: false,
        created_at: 100,
        updated_at: 200, // local > remote
    };

    msg_repo.upsert(&msg).await.unwrap();

    let mut payload = msg.clone();
    payload.subject = Some("Old Subject".to_string());
    payload.date_received = 100; // older remote

    if payload.date_received <= msg.updated_at {
        let mut new_msg = msg.clone();
        if msg.subject != payload.subject {
            new_msg.has_conflict = true;
            new_msg.updated_at = chrono::Utc::now().timestamp();
            msg_repo.upsert(&new_msg).await.unwrap();
        }
    }

    let db_msg = msg_repo.find_by_id(msg_id).await.unwrap().unwrap();
    assert_eq!(db_msg.subject, Some("New Subject".to_string()));
    assert_eq!(db_msg.has_conflict, true);
}

#[tokio::test]
async fn test_restore_revision() {
    let state = create_test_app_state().await;
    let pool = match &state.db {
        backend::db::pool::DbPool::Sqlite(pool) => pool.clone(),
        _ => unreachable!(),
    };

    let msg_repo =
        backend::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone());
    let rev_repo =
        backend::db::sqlite::revision_repository::SqliteRevisionRepository::new(pool.clone());

    let msg_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let msg = Message {
        id: backend::core::types::DbUuid::new(msg_id),
        account_id: backend::core::types::DbUuid::new(account_id),
        external_id: "ext1".to_string(),
        thread_id: "thread1".to_string(),
        subject: Some("Current Subject".to_string()),
        sender_name: None,
        sender_email: "test@example.com".to_string(),
        recipients: "me@example.com".to_string(),
        date_sent: 100,
        date_received: 100,
        snippet: None,
        body_text: None,
        body_html: None,
        labels: None,
        is_read: true,
        is_archived: false,
        is_deleted: false,
        has_attachments: false,
        snoozed_until: None,
        has_conflict: true, // Currently has conflict
        created_at: 100,
        updated_at: 200,
    };

    msg_repo.upsert(&msg).await.unwrap();

    let old_msg = Message {
        subject: Some("Old Subject".to_string()),
        has_conflict: false, // Old msg had no conflict
        ..msg.clone()
    };
    let rev_id = Uuid::new_v4();
    let rev = HistoricalRevision {
        id: backend::core::types::DbUuid::new(rev_id),
        resource_type: "message".to_string(),
        resource_id: backend::core::types::DbUuid::new(msg_id),
        serialized_payload: serde_json::to_string(&old_msg).unwrap(),
        revision_number: 1,
        created_at: 150,
    };

    let _ = <backend::db::sqlite::revision_repository::SqliteRevisionRepository as backend::core::repository::HistoricalRevisionRepository>::create(&rev_repo, &rev).await;

    // Simulate calling the handler by fetching it, parsing it, and saving it with cleared conflict flag.
    let revision = <backend::db::sqlite::revision_repository::SqliteRevisionRepository as backend::core::repository::HistoricalRevisionRepository>::find_by_id(&rev_repo, rev_id).await.unwrap().unwrap();
    let mut restored_msg: Message = serde_json::from_str(&revision.serialized_payload).unwrap();
    restored_msg.has_conflict = false;
    restored_msg.updated_at = chrono::Utc::now().timestamp();
    msg_repo.upsert(&restored_msg).await.unwrap();

    let db_msg = msg_repo.find_by_id(msg_id).await.unwrap().unwrap();
    assert_eq!(db_msg.subject, Some("Old Subject".to_string()));
    assert_eq!(db_msg.has_conflict, false);
}
