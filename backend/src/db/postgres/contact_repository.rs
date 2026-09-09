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
        .bind(contact.id.0)
        .bind(contact.account_id.0)
        .bind(&contact.name)
        .bind(&contact.email)
        .bind(&contact.avatar_url)
        .bind(contact.last_contacted_at)
        .bind(contact.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, account_id: Uuid, email: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM contacts WHERE account_id = $1 AND email = $2")
            .bind(account_id)
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn record_merge(
        &self,
        account_id: Uuid,
        keep_email: &str,
        loser_email: &str,
    ) -> Result<bool, sqlx::Error> {
        if keep_email.eq_ignore_ascii_case(loser_email) {
            return self.delete(account_id, loser_email).await;
        }
        sqlx::query(
            "INSERT INTO merged_contacts (account_id, email, merged_into) VALUES ($1, $2, $3)
             ON CONFLICT (account_id, email) DO UPDATE SET merged_into = EXCLUDED.merged_into",
        )
        .bind(account_id)
        .bind(loser_email)
        .bind(keep_email)
        .execute(&self.pool)
        .await?;
        self.delete(account_id, loser_email).await
    }

    async fn is_merged(&self, account_id: Uuid, email: &str) -> Result<bool, sqlx::Error> {
        let found: Option<String> = sqlx::query_scalar(
            "SELECT email FROM merged_contacts WHERE account_id = $1 AND email = $2",
        )
        .bind(account_id)
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(found.is_some())
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
