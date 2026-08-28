mod common;

use backend::core::models::{Account, CalendarEvent, Contact, User};
use backend::core::repository::{
    AccountRepository, CalendarRepository, ContactRepository, EventRepository, MessageRepository,
    UserPreferencesRepository, UserRepository,
};
use backend::core::types::DbUuid;
use backend::db::sqlite::account_repository::SqliteAccountRepository;
use backend::db::sqlite::calendar_repository::SqliteCalendarRepository;
use backend::db::sqlite::contact_repository::SqliteContactRepository;
use backend::db::sqlite::event_repository::SqliteEventRepository;
use backend::db::sqlite::message_repository::SqliteMessageRepository;
use backend::db::sqlite::user_preferences_repository::SqliteUserPreferencesRepository;
use backend::db::sqlite::user_repository::SqliteUserRepository;
use common::{
    get_sqlite_pool, seed_account, seed_calendar, seed_message, seed_user, setup_test_db,
};
use uuid::Uuid;

#[tokio::test]
async fn test_user_repository_crud() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteUserRepository::new(sqlite_pool);

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let user = User {
        id: DbUuid::from(user_id),
        username: "test@kestrel.dev".to_string(),
        password_hash: "argon2_hash_placeholder".to_string(),
        created_at: now,
        updated_at: now,
    };

    repo.create(&user).await.expect("User creation failed");

    let fetched = repo
        .find_by_username("test@kestrel.dev")
        .await
        .expect("Find by username failed");
    assert!(fetched.is_some());
    let created_user = fetched.unwrap();
    assert_eq!(created_user.username, "test@kestrel.dev");
    assert_eq!(created_user.id.0, user_id);
}

#[tokio::test]
async fn test_account_repository_crud() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let user_repo = SqliteUserRepository::new(sqlite_pool.clone());
    let account_repo =
        SqliteAccountRepository::new(sqlite_pool, "test_secret_32bytes_long_key!!".to_string());

    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let user_model = User {
        id: DbUuid::from(user_id),
        username: "account_owner@kestrel.dev".to_string(),
        password_hash: "hash".to_string(),
        created_at: now,
        updated_at: now,
    };
    user_repo
        .create(&user_model)
        .await
        .expect("User creation failed");

    let account_id = Uuid::new_v4();
    let account = Account {
        id: DbUuid::from(account_id),
        user_id: DbUuid::from(user_id),
        provider: "gmail".to_string(),
        provider_account_id: "google_12345".to_string(),
        display_name: "Owner".to_string(),
        access_token: Some("access_token_123".to_string()),
        refresh_token: Some("refresh_token_123".to_string()),
        token_expires_at: Some(now + 3600),
        sync_error: None,
        created_at: now,
        updated_at: now,
    };

    account_repo
        .create(&account)
        .await
        .expect("Account creation failed");

    let list = account_repo
        .find_by_user_id(user_id)
        .await
        .expect("Account list failed");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].display_name, "Owner");
    assert_eq!(list[0].provider, "gmail");

    let fetched = account_repo
        .find_by_id(account_id)
        .await
        .expect("Find by id failed");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().provider_account_id, "google_12345");

    account_repo
        .delete(account_id)
        .await
        .expect("Account delete failed");
    let after_del = account_repo
        .find_by_user_id(user_id)
        .await
        .expect("Account list failed");
    assert_eq!(after_del.len(), 0);
}

#[tokio::test]
async fn test_message_repository_crud_and_queries() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteMessageRepository::new(sqlite_pool);

    let user = seed_user(&pool, "msg_user@kestrel.dev").await;
    let account = seed_account(&pool, user.id.0, "gmail", "Work Mail").await;

    // 1. Insert messages
    let msg1 = seed_message(
        &pool,
        account.id.0,
        "Meeting notes",
        "boss@corp.com",
        "Here are the notes",
        Some("inbox"),
        false,
    )
    .await;
    let msg2 = seed_message(
        &pool,
        account.id.0,
        "Newsletter #42",
        "news@daily.com",
        "Weekly updates inside",
        Some("newsletter"),
        true,
    )
    .await;

    // 2. Find by ID
    let found = repo.find_by_id(msg1.id.0).await.expect("find_by_id failed");
    assert!(found.is_some());
    let found_msg = found.unwrap();
    assert_eq!(found_msg.subject, Some("Meeting notes".to_string()));
    assert_eq!(found_msg.sender_email, "boss@corp.com");
    assert_eq!(found_msg.is_read, false);

    // 3. List messages with account_id filter
    let list = repo
        .list(Some(account.id.0), None, None, 10)
        .await
        .expect("list failed");
    assert_eq!(list.len(), 2);

    // 4. Mark as read
    repo.set_read(msg1.id.0, true)
        .await
        .expect("set_read failed");
    let updated = repo
        .find_by_id(msg1.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(updated.is_read, true);

    // 5. Archive message
    repo.set_archived(msg1.id.0, true)
        .await
        .expect("set_archived failed");
    let archived = repo
        .find_by_id(msg1.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(archived.is_archived, true);

    // 6. Delete message (soft delete)
    repo.set_deleted(msg2.id.0, true)
        .await
        .expect("set_deleted failed");
    let deleted = repo
        .find_by_id(msg2.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(deleted.is_deleted, true);
}

#[tokio::test]
async fn test_message_repository_labels_and_snooze() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteMessageRepository::new(sqlite_pool);

    let user = seed_user(&pool, "snooze_user@kestrel.dev").await;
    let account = seed_account(&pool, user.id.0, "outlook", "Outlook Account").await;
    let msg = seed_message(
        &pool,
        account.id.0,
        "Urgent Task",
        "team@corp.com",
        "Please review ASAP",
        Some("inbox"),
        false,
    )
    .await;

    // Set labels
    repo.set_labels(msg.id.0, Some("inbox,starred,work".to_string()))
        .await
        .expect("set_labels failed");
    let updated = repo
        .find_by_id(msg.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(updated.labels, Some("inbox,starred,work".to_string()));

    // Snooze message
    let snooze_time = chrono::Utc::now().timestamp() + 300;
    repo.set_snoozed_until(msg.id.0, Some(snooze_time))
        .await
        .expect("set_snoozed_until failed");
    let snoozed = repo
        .find_by_id(msg.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(snoozed.snoozed_until, Some(snooze_time));

    // Unsnooze query: check when current timestamp exceeds snooze_time
    let due = repo
        .unsnooze_due_messages(snooze_time + 10)
        .await
        .expect("unsnooze_due_messages failed");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0], msg.id.0);

    // Verify it is no longer snoozed
    let unsnoozed = repo
        .find_by_id(msg.id.0)
        .await
        .expect("find failed")
        .unwrap();
    assert_eq!(unsnoozed.snoozed_until, None);
}

#[tokio::test]
async fn test_calendar_and_event_repository() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let cal_repo = SqliteCalendarRepository::new(sqlite_pool.clone());
    let ev_repo = SqliteEventRepository::new(sqlite_pool);

    let user = seed_user(&pool, "cal_user@kestrel.dev").await;
    let account = seed_account(&pool, user.id.0, "google", "Google Calendar").await;

    // 1. Create calendar
    let cal = seed_calendar(&pool, account.id.0, "Personal", true).await;
    let found_cal = cal_repo
        .find_by_id(cal.id.0)
        .await
        .expect("find_by_id failed");
    assert!(found_cal.is_some());
    assert_eq!(found_cal.unwrap().name, "Personal");

    let user_cals = cal_repo
        .list_by_user(user.id.0)
        .await
        .expect("list_by_user failed");
    assert_eq!(user_cals.len(), 1);

    // 2. Create events
    let now = chrono::Utc::now().timestamp();
    let ev_id = Uuid::new_v4();
    let ev = CalendarEvent {
        id: DbUuid::from(ev_id),
        account_id: DbUuid::from(account.id.0),
        calendar_id: DbUuid::from(cal.id.0),
        external_id: format!("ev_{}", ev_id),
        title: "Team Standup".to_string(),
        description: Some("Daily Sync".to_string()),
        location: Some("Virtual".to_string()),
        start_time: now + 3600,
        end_time: now + 5400,
        is_all_day: false,
        recurrence_rules: None,
        organizer_email: Some("lead@kestrel.dev".to_string()),
        organizer_name: Some("Tech Lead".to_string()),
        attendees: Some(
            r#"[{"name":"Dev","email":"dev@kestrel.dev","status":"accepted"}]"#.to_string(),
        ),
        status: Some("confirmed".to_string()),
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    ev_repo.upsert(&ev).await.expect("upsert event failed");

    // 3. Find event by ID
    let found_ev = ev_repo.find_by_id(ev_id).await.expect("find_by_id failed");
    assert!(found_ev.is_some());
    let unwrapped_ev = found_ev.unwrap();
    assert_eq!(unwrapped_ev.title, "Team Standup");
    assert_eq!(unwrapped_ev.start_time, now + 3600);

    // 4. Query events by range
    let range_events = ev_repo
        .list_range(user.id.0, now, now + 7200, Some(cal.id.0))
        .await
        .expect("list_range failed");
    assert_eq!(range_events.len(), 1);
    assert_eq!(range_events[0].title, "Team Standup");

    // Range outside bounds should return 0
    let out_range = ev_repo
        .list_range(user.id.0, now + 10000, now + 20000, None)
        .await
        .expect("list_range failed");
    assert_eq!(out_range.len(), 0);

    // 5. Soft delete event
    ev_repo
        .soft_delete(ev_id)
        .await
        .expect("soft_delete failed");
    let after_del = ev_repo.find_by_id(ev_id).await.expect("find failed");
    assert_eq!(after_del.unwrap().title, "[deleted]");
}

#[tokio::test]
async fn test_contact_repository_upsert_and_search() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteContactRepository::new(sqlite_pool);

    let user = seed_user(&pool, "contact_user@kestrel.dev").await;
    let account = seed_account(&pool, user.id.0, "gmail", "Personal").await;

    let now = chrono::Utc::now().timestamp();
    let contact1 = Contact {
        id: DbUuid::from(Uuid::new_v4()),
        account_id: DbUuid::from(account.id.0),
        name: Some("Alice Smith".to_string()),
        email: "alice.smith@example.com".to_string(),
        avatar_url: None,
        last_contacted_at: now,
        created_at: now,
    };
    let contact2 = Contact {
        id: DbUuid::from(Uuid::new_v4()),
        account_id: DbUuid::from(account.id.0),
        name: Some("Bob Johnson".to_string()),
        email: "bob.j@example.com".to_string(),
        avatar_url: None,
        last_contacted_at: now,
        created_at: now,
    };

    repo.upsert(&contact1)
        .await
        .expect("upsert contact1 failed");
    repo.upsert(&contact2)
        .await
        .expect("upsert contact2 failed");

    // Search by name prefix
    let results = repo
        .search(&[account.id.0], "Ali", 10)
        .await
        .expect("search failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].email, "alice.smith@example.com");

    // Search by email prefix
    let results_email = repo
        .search(&[account.id.0], "bob", 10)
        .await
        .expect("search failed");
    assert_eq!(results_email.len(), 1);
    assert_eq!(results_email[0].name, Some("Bob Johnson".to_string()));
}

#[tokio::test]
async fn test_user_preferences_repository() {
    let pool = setup_test_db().await;
    let sqlite_pool = get_sqlite_pool(&pool);
    let repo = SqliteUserPreferencesRepository::new(sqlite_pool);

    let user = seed_user(&pool, "pref_user@kestrel.dev").await;
    let user_id = user.id.0;

    // Initial query should return None
    let initial = repo
        .get_preferences(user_id)
        .await
        .expect("get_preferences failed");
    assert!(initial.is_none());

    // Update preferences
    let prefs_json = r#"{"mailDenseMode":true,"syncInterval":300}"#;
    repo.update_preferences(user_id, prefs_json)
        .await
        .expect("update_preferences failed");

    // Query should return stored preferences
    let fetched = repo
        .get_preferences(user_id)
        .await
        .expect("get_preferences failed");
    assert!(fetched.is_some());
    let prefs = fetched.unwrap();
    assert_eq!(prefs.preferences_json, prefs_json);
}
