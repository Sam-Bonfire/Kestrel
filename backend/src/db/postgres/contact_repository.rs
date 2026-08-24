use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::Contact;
use crate::core::repository::ContactRepository;

pub struct PostgresContactRepository {
    pool: PgPool,
}

impl PostgresContactRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ContactRepository for PostgresContactRepository {
    async fn upsert(&self, contact: &Contact) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO contacts (id, account_id, name, email, avatar_url, last_contacted_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (account_id, email) DO UPDATE SET
             name = COALESCE(EXCLUDED.name, contacts.name),
             avatar_url = COALESCE(EXCLUDED.avatar_url, contacts.avatar_url),
             last_contacted_at = GREATEST(contacts.last_contacted_at, EXCLUDED.last_contacted_at)",
        )
        .bind(&contact.id.0)
        .bind(&contact.account_id.0)
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

        let query_str = format!("{}%", query);

        let sql = "SELECT id, account_id, name, email, avatar_url, last_contacted_at, created_at
             FROM contacts
             WHERE account_id = ANY($1) AND (email ILIKE $2 OR name ILIKE $2)
             ORDER BY last_contacted_at DESC
             LIMIT $3";

        let contacts = sqlx::query_as::<_, Contact>(sql)
            .bind(account_ids)
            .bind(&query_str)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(contacts)
    }
}
