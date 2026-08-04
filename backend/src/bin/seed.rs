use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::seq::SliceRandom;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use uuid::Uuid;

const SENDERS: &[(&str, &str)] = &[
    ("Alice Smith", "alice@example.com"),
    ("Bob Jones", "bob.jones@workplace.net"),
    ("Charlie Brown", "charlie@peanuts.com"),
    ("Diana Prince", "diana@themyscira.gov"),
    ("Eve Polastri", "eve@mi6.gov.uk"),
    ("System", "noreply@system.local"),
];

const SUBJECTS: &[&str] = &[
    "Project Update: Phase 2",
    "Lunch today?",
    "Invoice #10294",
    "Welcome to the team!",
    "Weekly Sync Minutes",
    "URGENT: Server Down",
    "Happy Birthday!",
    "Q3 Financial Report",
    "Your recent order",
];

const BODIES: &[&str] = &[
    "<p>Hi there,</p><p>Just wanted to touch base regarding our upcoming deadline. Let's sync tomorrow.</p><p>Best,</p>",
    "<p>Are we still on for lunch at 12:30?</p>",
    "<p>Please find the attached invoice. Let me know if you have any questions.</p>",
    "<p>I'm so excited to start working with you all!</p>",
    "<p><strong>Action Items:</strong></p><ul><li>Fix the bug</li><li>Deploy to prod</li></ul>",
    "<p>The server is currently experiencing downtime. We are investigating.</p>",
    "<p>Hope you have a fantastic birthday!</p>",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        format!("sqlite:{}/data/kestrel.db", manifest_dir)
    });
    println!("Connecting to database at {}", db_url);

    let pool = SqlitePoolOptions::new().connect(&db_url).await?;

    println!("Generating fake data...");

    // Create a dummy user
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"password", &salt)
        .expect("Failed to hash password")
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO users (id, username, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(username) DO NOTHING
        "#,
    )
    .bind(&user_id)
    .bind("demo_user")
    .bind(&hash)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    // Fetch the actual user_id in case it already existed
    let row: (String,) = sqlx::query_as("SELECT id FROM users WHERE username = 'demo_user'")
        .fetch_one(&pool)
        .await?;
    let user_id = row.0;

    // Create a dummy account
    let account_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO accounts (id, user_id, provider, provider_account_id, display_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(user_id, provider, provider_account_id) DO NOTHING
        "#
    )
    .bind(&account_id)
    .bind(&user_id)
    .bind("mock")
    .bind("demo@kestrel.local")
    .bind("Local Demo Account")
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    let row: (String,) =
        sqlx::query_as("SELECT id FROM accounts WHERE user_id = ? AND provider = 'mock'")
            .bind(&user_id)
            .fetch_one(&pool)
            .await?;
    let account_id = row.0;

    println!("Using user_id: {} and account_id: {}", user_id, account_id);

    // Wipe existing mock messages for this account to prevent endless buildup
    sqlx::query("DELETE FROM messages WHERE account_id = ?")
        .bind(&account_id)
        .execute(&pool)
        .await?;

    let mut rng = rand::thread_rng();

    // Insert 100 fake emails
    for i in 0..100 {
        let msg_id = Uuid::new_v4().to_string();
        let thread_id = if rng.gen_bool(0.3) {
            Uuid::new_v4().to_string()
        } else {
            msg_id.clone()
        };

        let (sender_name, sender_email) = SENDERS.choose(&mut rng).unwrap();
        let subject = SUBJECTS.choose(&mut rng).unwrap();
        let body = BODIES.choose(&mut rng).unwrap();

        let days_ago = rng.gen_range(0..30);
        let ts = (Utc::now() - Duration::days(days_ago)).timestamp();

        let is_read = rng.gen_bool(0.7);
        let is_archived = rng.gen_bool(0.2);
        let is_deleted = rng.gen_bool(0.05);
        let has_attachments = rng.gen_bool(0.1);
        let snoozed_until = if rng.gen_bool(0.05) {
            Some(now + 86400)
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO messages (
                id, account_id, external_id, thread_id, subject, 
                sender_name, sender_email, recipients, date_sent, date_received, 
                snippet, body_text, body_html, labels, is_read, is_archived, is_deleted, has_attachments, snoozed_until
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, external_id) DO NOTHING
            "#
        )
        .bind(&msg_id)
        .bind(&account_id)
        .bind(format!("mock-ext-{}", msg_id))
        .bind(&thread_id)
        .bind(subject)
        .bind(sender_name)
        .bind(sender_email)
        .bind("demo@kestrel.local")
        .bind(ts)
        .bind(ts)
        .bind(subject) // Snippet
        .bind("") 
        .bind(body)
        .bind("[]") // Labels
        .bind(is_read)
        .bind(is_archived)
        .bind(is_deleted)
        .bind(has_attachments)
        .bind(snoozed_until)
        .execute(&pool)
        .await?;
    }

    // Generate some calendar events
    sqlx::query("DELETE FROM calendars WHERE account_id = ?")
        .bind(&account_id)
        .execute(&pool)
        .await?;
    sqlx::query("DELETE FROM calendar_events WHERE account_id = ?")
        .bind(&account_id)
        .execute(&pool)
        .await?;

    let calendar_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO calendars (id, account_id, external_id, name, color, is_primary, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(&calendar_id)
    .bind(&account_id)
    .bind("mock-cal-1")
    .bind("Work Calendar")
    .bind("#4285F4")
    .bind(true)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    for i in 0..20 {
        let event_id = Uuid::new_v4().to_string();
        let days_offset = rng.gen_range(-10..10);
        let start_time = (Utc::now() + Duration::days(days_offset)).timestamp();
        let end_time = start_time + 3600; // 1 hour duration

        let title = format!("Meeting {}", i);

        sqlx::query(
            r#"
            INSERT INTO calendar_events (
                id, account_id, calendar_id, external_id, title, description, location, 
                start_time, end_time, is_all_day, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&event_id)
        .bind(&account_id)
        .bind(&calendar_id)
        .bind(format!("mock-event-{}", event_id))
        .bind(&title)
        .bind("Discuss project updates")
        .bind("Conference Room B")
        .bind(start_time)
        .bind(end_time)
        .bind(false)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await?;
    }

    println!("Seeding complete! 100 emails and 20 calendar events inserted for demo_user.");

    Ok(())
}
