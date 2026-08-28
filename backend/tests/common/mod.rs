use backend::core::models::{Account, Calendar, CalendarEvent, Contact, Message, User};
use backend::core::repository::{
    AccountRepository, CalendarRepository, ContactRepository, EventRepository, MessageRepository,
    UserRepository,
};
use backend::core::types::DbUuid;
use backend::db::pool::{DbPool, run_migrations};
use backend::db::sqlite::account_repository::SqliteAccountRepository;
use backend::db::sqlite::calendar_repository::SqliteCalendarRepository;
use backend::db::sqlite::contact_repository::SqliteContactRepository;
use backend::db::sqlite::event_repository::SqliteEventRepository;
use backend::db::sqlite::message_repository::SqliteMessageRepository;
use backend::db::sqlite::user_repository::SqliteUserRepository;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn setup_test_db() -> DbPool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to in-memory SQLite database");

    let db_pool = DbPool::Sqlite(pool.clone());
    run_migrations(&db_pool)
        .await
        .expect("Failed to run migrations for test DB");
    let _ = sqlx::query("ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;")
        .execute(&pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;",
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS historical_revisions (id TEXT PRIMARY KEY NOT NULL, resource_type TEXT NOT NULL, resource_id TEXT NOT NULL, serialized_payload TEXT NOT NULL, revision_number INTEGER NOT NULL, created_at INTEGER NOT NULL DEFAULT (unixepoch()));").execute(&pool).await;
    db_pool
}

pub const TEST_SECRET: &str = "test_secret_32bytes_long_key!!";

pub fn get_sqlite_pool(pool: &DbPool) -> SqlitePool {
    match pool {
        DbPool::Sqlite(p) => p.clone(),
        _ => panic!("Expected SQLite pool in test suite"),
    }
}

pub async fn seed_user(pool: &DbPool, username: &str) -> User {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteUserRepository::new(sqlite);
    let now = chrono::Utc::now().timestamp();
    let user = User {
        id: DbUuid::from(Uuid::new_v4()),
        username: username.to_string(),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$placeholder_hash".to_string(),
        created_at: now,
        updated_at: now,
    };
    repo.create(&user).await.expect("Failed to seed user");
    user
}

pub async fn seed_account(
    pool: &DbPool,
    user_id: Uuid,
    provider: &str,
    display_name: &str,
) -> Account {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteAccountRepository::new(sqlite, TEST_SECRET.to_string());
    let now = chrono::Utc::now().timestamp();
    let account = Account {
        id: DbUuid::from(Uuid::new_v4()),
        user_id: DbUuid::from(user_id),
        provider: provider.to_string(),
        provider_account_id: format!("{}_{}", provider, Uuid::new_v4()),
        display_name: display_name.to_string(),
        access_token: Some("mock_access_token_123".to_string()),
        refresh_token: Some("mock_refresh_token_123".to_string()),
        token_expires_at: Some(now + 3600),
        sync_error: None,
        created_at: now,
        updated_at: now,
    };
    repo.create(&account).await.expect("Failed to seed account");
    account
}

pub async fn seed_message(
    pool: &DbPool,
    account_id: Uuid,
    subject: &str,
    sender_email: &str,
    body_text: &str,
    labels: Option<&str>,
    is_read: bool,
) -> Message {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteMessageRepository::new(sqlite);
    let now = chrono::Utc::now().timestamp();
    let msg_id = Uuid::new_v4();
    let msg = Message {
        id: DbUuid::from(msg_id),
        account_id: DbUuid::from(account_id),
        external_id: format!("ext_{}", msg_id),
        thread_id: format!("th_{}", msg_id),
        subject: Some(subject.to_string()),
        sender_name: Some("Sender Name".to_string()),
        sender_email: sender_email.to_string(),
        recipients: "user@kestrel.dev".to_string(),
        date_sent: now - 60,
        date_received: now,
        snippet: Some(body_text.chars().take(50).collect()),
        body_text: Some(body_text.to_string()),
        body_html: Some(format!("<p>{}</p>", body_text)),
        labels: labels.map(|l| l.to_string()),
        is_read,
        is_archived: false,
        is_deleted: false,
        has_attachments: false,
        snoozed_until: None,
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    repo.upsert(&msg).await.expect("Failed to seed message");
    msg
}

pub async fn seed_calendar(
    pool: &DbPool,
    account_id: Uuid,
    name: &str,
    is_primary: bool,
) -> Calendar {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteCalendarRepository::new(sqlite);
    let now = chrono::Utc::now().timestamp();
    let cal_id = Uuid::new_v4();
    let cal = Calendar {
        id: DbUuid::from(cal_id),
        account_id: DbUuid::from(account_id),
        external_id: format!("cal_ext_{}", cal_id),
        name: name.to_string(),
        color: Some("#4F46E5".to_string()),
        is_primary,
        created_at: now,
        updated_at: now,
    };
    repo.upsert(&cal).await.expect("Failed to seed calendar");
    cal
}

pub async fn seed_event(
    pool: &DbPool,
    account_id: Uuid,
    calendar_id: Uuid,
    title: &str,
    start_time: i64,
    end_time: i64,
) -> CalendarEvent {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteEventRepository::new(sqlite);
    let now = chrono::Utc::now().timestamp();
    let ev_id = Uuid::new_v4();
    let ev = CalendarEvent {
        id: DbUuid::from(ev_id),
        account_id: DbUuid::from(account_id),
        calendar_id: DbUuid::from(calendar_id),
        external_id: format!("ev_ext_{}", ev_id),
        title: title.to_string(),
        description: Some("Event Description".to_string()),
        location: Some("Meeting Room A".to_string()),
        start_time,
        end_time,
        is_all_day: false,
        recurrence_rules: None,
        organizer_email: Some("organizer@kestrel.dev".to_string()),
        organizer_name: Some("Organizer".to_string()),
        attendees: Some(
            r#"[{"email":"attendee@kestrel.dev","name":"Attendee","status":"accepted"}]"#
                .to_string(),
        ),
        status: Some("confirmed".to_string()),
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    repo.upsert(&ev).await.expect("Failed to seed event");
    ev
}

pub async fn seed_contact(pool: &DbPool, account_id: Uuid, name: &str, email: &str) -> Contact {
    let sqlite = get_sqlite_pool(pool);
    let repo = SqliteContactRepository::new(sqlite);
    let now = chrono::Utc::now().timestamp();
    let contact = Contact {
        id: DbUuid::from(Uuid::new_v4()),
        account_id: DbUuid::from(account_id),
        name: Some(name.to_string()),
        email: email.to_string(),
        avatar_url: None,
        last_contacted_at: now,
        created_at: now,
    };
    repo.upsert(&contact).await.expect("Failed to seed contact");
    contact
}
