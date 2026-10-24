use backend::core::models::{Account, Calendar, CalendarEvent, User};
use backend::core::repository::{
    AccountRepository, CalendarRepository, EventPollRepository, EventRepository, UserRepository,
};
use backend::core::types::DbUuid;
use backend::db::pool::{DbPool, init_pool, run_migrations};
use backend::db::postgres::account_repository::PostgresAccountRepository;
use backend::db::postgres::calendar_repository::PostgresCalendarRepository;
use backend::db::postgres::event_repository::PostgresEventRepository;
use backend::db::postgres::poll_repository::PostgresPollRepository;
use backend::db::postgres::user_repository::PostgresUserRepository;
use sqlx::PgPool;
use uuid::Uuid;

async fn postgres_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    if !url.starts_with("postgres://") && !url.starts_with("postgresql://") {
        eprintln!("SKIP postgres contract test: DATABASE_URL is not a Postgres URL");
        return None;
    }

    let db = init_pool(&url).await.expect("connect to Postgres");
    run_migrations(&db).await.expect("run Postgres migrations");
    match db {
        DbPool::Postgres(pool) => Some(pool),
        DbPool::Sqlite(_) => unreachable!("DATABASE_URL checked above"),
    }
}

#[tokio::test]
async fn postgres_jsonb_round_trips_for_events_and_polls() {
    let Some(pool) = postgres_pool().await else {
        return;
    };

    let now = chrono::Utc::now().timestamp();
    let user = User {
        id: DbUuid::from(Uuid::new_v4()),
        username: format!("contract_{}@kestrel.dev", Uuid::new_v4()),
        password_hash: "$argon2id$v=19$placeholder".to_string(),
        created_at: now,
        updated_at: now,
    };
    PostgresUserRepository::new(pool.clone())
        .create(&user)
        .await
        .expect("create user");

    let account = Account {
        id: DbUuid::from(Uuid::new_v4()),
        user_id: user.id,
        provider: "gmail".to_string(),
        provider_account_id: format!("contract_{}", Uuid::new_v4()),
        display_name: "Contract Account".to_string(),
        access_token: Some("token".to_string()),
        refresh_token: Some("refresh".to_string()),
        token_expires_at: Some(now + 3600),
        sync_error: None,
        created_at: now,
        updated_at: now,
    };
    PostgresAccountRepository::new(
        pool.clone(),
        "contract_secret_32bytes_long_key!!".to_string(),
    )
    .create(&account)
    .await
    .expect("create account");

    let calendar = Calendar {
        id: DbUuid::from(Uuid::new_v4()),
        account_id: account.id,
        external_id: format!("cal_{}", Uuid::new_v4()),
        name: "Contract Calendar".to_string(),
        color: None,
        is_primary: true,
        created_at: now,
        updated_at: now,
    };
    PostgresCalendarRepository::new(pool.clone())
        .upsert(&calendar)
        .await
        .expect("create calendar");

    let event_repo = PostgresEventRepository::new(pool.clone());
    let attendees = r#"[{"email":"guest@example.com","name":"Guest"}]"#;
    let event = CalendarEvent {
        id: DbUuid::from(Uuid::new_v4()),
        account_id: account.id,
        calendar_id: calendar.id,
        external_id: format!("event_{}", Uuid::new_v4()),
        title: "Contract Event".to_string(),
        description: None,
        location: None,
        start_time: now,
        end_time: now + 3600,
        is_all_day: false,
        recurrence_rules: None,
        organizer_email: Some("owner@example.com".to_string()),
        organizer_name: None,
        attendees: Some(attendees.to_string()),
        status: Some("confirmed".to_string()),
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    event_repo.upsert(&event).await.expect("upsert event");

    let fetched = event_repo
        .find_by_id(event.id.0)
        .await
        .expect("find event")
        .expect("event exists");
    let stored: Vec<serde_json::Value> =
        serde_json::from_str(fetched.attendees.as_deref().unwrap_or("")).expect("attendees JSON");
    assert_eq!(stored[0]["email"], "guest@example.com");
    assert!(fetched.has_conflict == false);

    let poll_repo = PostgresPollRepository::new(pool.clone());
    let poll = poll_repo
        .create_poll(
            event.id.0,
            "Which slot?",
            &["Monday".to_string(), "Tuesday".to_string()],
        )
        .await
        .expect("create poll");

    let polls = poll_repo.list_polls(event.id.0).await.expect("list polls");
    assert_eq!(polls.len(), 1);
    assert_eq!(polls[0].id.0, poll.id.0);
    assert_eq!(polls[0].options, vec!["Monday", "Tuesday"]);

    sqlx::query("DELETE FROM event_polls WHERE event_id = $1")
        .bind(event.id.0)
        .execute(&pool)
        .await
        .expect("cleanup poll");
    sqlx::query("DELETE FROM calendar_events WHERE id = $1")
        .bind(event.id.0)
        .execute(&pool)
        .await
        .expect("cleanup event");
}
