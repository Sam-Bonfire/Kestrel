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
            "INSERT INTO contacts (id, account_id, name, email, avatar_url, last_contacted_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, email) DO UPDATE SET
             name = COALESCE(excluded.name, contacts.name),
             avatar_url = COALESCE(excluded.avatar_url, contacts.avatar_url),
             last_contacted_at = excluded.last_contacted_at",
        )
        .bind(contact.id.to_string())
        .bind(contact.account_id.to_string())
        .bind(&contact.name)
        .bind(&contact.email)
        .bind(&contact.avatar_url)
        .bind(contact.last_contacted_at)
        .bind(contact.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
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
            "SELECT id, account_id, name, email, avatar_url, last_contacted_at, created_at
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
