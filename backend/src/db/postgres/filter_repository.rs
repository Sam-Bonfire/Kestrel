use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::repository::FilterRepository;

pub struct PostgresFilterRepository {
    pool: PgPool,
}

impl PostgresFilterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FilterRepository for PostgresFilterRepository {
    async fn block_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO blocked_senders (user_id, email_address)
             VALUES ($1, $2)
             ON CONFLICT(user_id, email_address) DO NOTHING"
        )
        .bind(user_id)
        .bind(email)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_blocked_senders(&self, user_id: Uuid) -> Result<Vec<String>, sqlx::Error> {
        let records = sqlx::query_scalar::<_, String>(
            "SELECT email_address FROM blocked_senders WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(records)
    }
}
