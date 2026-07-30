use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::repository::FilterRepository;

pub struct SqliteFilterRepository {
    pool: SqlitePool,
}

impl SqliteFilterRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FilterRepository for SqliteFilterRepository {
    async fn block_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO blocked_senders (id, user_id, email_address)
             VALUES (?, ?, ?)
             ON CONFLICT(user_id, email_address) DO NOTHING"
        )
        .bind(id)
        .bind(user_id.to_string())
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_blocked_senders(&self, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        let records = sqlx::query_scalar::<_, String>(
            "SELECT email_address FROM blocked_senders WHERE user_id = ?"
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }
}
