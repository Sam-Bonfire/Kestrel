use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use std::env;
use std::fs;
use uuid::Uuid;
use chrono::DateTime;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct MockEmail {
    id: String,
    sender: String,
    senderEmail: String,
    to: String,
    subject: String,
    timestamp: String,
    isUnread: bool,
    isArchived: bool,
    isStarred: bool,
    isDraft: bool,
    isSpam: bool,
    isTrash: bool,
    hasAttachment: bool,
    category: String,
    labels: Vec<String>,
    avatar: String,
    body: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://kestrel.db".to_string());
    println!("Connecting to database at {}", db_url);
    
    let pool = SqlitePoolOptions::new()
        .connect(&db_url)
        .await?;

    println!("Reading seed.json...");
    let file_content = fs::read_to_string("data/seed.json")?;
    let emails: Vec<MockEmail> = serde_json::from_str(&file_content)?;

    println!("Found {} mock emails.", emails.len());

    // Create a dummy user
    let user_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    
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
        "#
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
    .bind("gmail")
    .bind("user@workspace.io")
    .bind("Workspace Mail")
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await?;

    let row: (String,) = sqlx::query_as("SELECT id FROM accounts WHERE user_id = ? AND provider = 'gmail'")
        .bind(&user_id)
        .fetch_one(&pool)
        .await?;
    let account_id = row.0;

    println!("Using user_id: {} and account_id: {}", user_id, account_id);

    // Insert emails
    for email in emails {
        let msg_id = Uuid::new_v4().to_string();
        
        let ts = DateTime::parse_from_rfc3339(&email.timestamp)
            .map(|dt| dt.timestamp())
            .unwrap_or(now);

        let labels_json = serde_json::to_string(&email.labels).unwrap_or_else(|_| "[]".to_string());
        
        // Map states
        let is_read = !email.isUnread;
        let is_deleted = email.isTrash;
        let is_archived = email.isArchived;
        
        // The trigger requires sender_name to be passed since we use it in fts.
        sqlx::query(
            r#"
            INSERT INTO messages (
                id, account_id, external_id, thread_id, subject, 
                sender_name, sender_email, recipients, date_sent, date_received, 
                snippet, body_text, body_html, labels, is_read, is_archived, is_deleted, has_attachments
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(account_id, external_id) DO NOTHING
            "#
        )
        .bind(&msg_id)
        .bind(&account_id)
        .bind(&email.id)
        .bind(&email.id) // using external_id as thread_id for mock
        .bind(&email.subject)
        .bind(&email.sender)
        .bind(&email.senderEmail)
        .bind(&email.to)
        .bind(ts)
        .bind(ts)
        .bind(&email.subject) // using subject as snippet
        .bind("") // no plain text body in mock
        .bind(&email.body)
        .bind(&labels_json)
        .bind(is_read)
        .bind(is_archived)
        .bind(is_deleted)
        .bind(email.hasAttachment)
        .execute(&pool)
        .await?;
    }

    println!("Seeding complete!");

    Ok(())
}
