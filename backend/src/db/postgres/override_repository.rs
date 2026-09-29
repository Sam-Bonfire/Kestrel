use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::ThreadSubjectOverride;
use crate::core::repository::ThreadSubjectOverrideRepository;

pub struct PostgresThreadSubjectOverrideRepository {
    pool: PgPool,
}

impl PostgresThreadSubjectOverrideRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ThreadSubjectOverrideRepository for PostgresThreadSubjectOverrideRepository {
    async fn upsert(
        &self,
        account_id: Uuid,
        thread_id: &str,
        subject: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO thread_subject_overrides (account_id, thread_id, subject, updated_at) \
             VALUES ($1, $2, $3, EXTRACT(EPOCH FROM NOW())::BIGINT) \
             ON CONFLICT (account_id, thread_id) DO UPDATE SET \
             subject = EXCLUDED.subject, updated_at = EXCLUDED.updated_at",
        )
        .bind(account_id)
        .bind(thread_id)
        .bind(subject)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_account(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<ThreadSubjectOverride>, sqlx::Error> {
        sqlx::query_as::<_, ThreadSubjectOverride>(
            "SELECT account_id, thread_id, subject, updated_at \
             FROM thread_subject_overrides WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, account_id: Uuid, thread_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "DELETE FROM thread_subject_overrides WHERE account_id = $1 AND thread_id = $2",
        )
        .bind(account_id)
        .bind(thread_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
