use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::Contact;
use crate::core::repository::ContactRepository;

pub struct SqliteContactRepository {
    pool: SqlitePool,
}

impl SqliteContactRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContactRepository for SqliteContactRepository {
    async fn upsert(&self, contact: &Contact) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO contacts (id, account_id, name, email, avatar_url, notes, last_contacted_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, email) DO UPDATE SET
             name = COALESCE(excluded.name, contacts.name),
             avatar_url = COALESCE(excluded.avatar_url, contacts.avatar_url),
             notes = COALESCE(excluded.notes, contacts.notes),
             last_contacted_at = excluded.last_contacted_at",
        )
        .bind(contact.id.to_string())
        .bind(contact.account_id.to_string())
        .bind(&contact.name)
        .bind(&contact.email)
        .bind(&contact.avatar_url)
        .bind(&contact.notes)
        .bind(contact.last_contacted_at)
        .bind(contact.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_notes(
        &self,
        account_id: Uuid,
        email: &str,
        notes: &str,
    ) -> Result<bool, sqlx::Error> {
        // Upsert: notes must be savable even before any sync has seen the sender.
        let now = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO contacts (id, account_id, email, notes, last_contacted_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, email) DO UPDATE SET notes = excluded.notes",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(account_id.to_string())
        .bind(email)
        .bind(notes)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    async fn search(
        &self,
        account_ids: &[Uuid],
        query: &str,
        limit: i64,
    ) -> Result<Vec<Contact>, sqlx::Error> {
        if account_ids.is_empty() {
            return Ok(vec![]);
        }

        let account_ids_str: Vec<String> = account_ids.iter().map(|id| id.to_string()).collect();
        let query_str = format!("{}%", query);

        let in_clause = vec!["?"; account_ids.len()].join(", ");

        let sql = format!(
            "SELECT id, account_id, name, email, avatar_url, notes, last_contacted_at, created_at
             FROM contacts
             WHERE account_id IN ({}) AND (email LIKE ? OR name LIKE ?)
             ORDER BY last_contacted_at DESC
             LIMIT ?",
            in_clause
        );

        let mut q = sqlx::query_as::<_, Contact>(&sql);

        for id in &account_ids_str {
            q = q.bind(id);
        }

        q = q.bind(&query_str).bind(&query_str).bind(limit);

        let contacts = q.fetch_all(&self.pool).await?;

        Ok(contacts)
    }
}
