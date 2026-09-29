use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::ThreadSubjectOverride;
use crate::core::repository::ThreadSubjectOverrideRepository;

pub struct SqliteThreadSubjectOverrideRepository {
    pool: SqlitePool,
}

impl SqliteThreadSubjectOverrideRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ThreadSubjectOverrideRepository for SqliteThreadSubjectOverrideRepository {
    async fn upsert(
        &self,
        account_id: Uuid,
        thread_id: &str,
        subject: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO thread_subject_overrides (account_id, thread_id, subject, updated_at) \
             VALUES (?, ?, ?, unixepoch()) \
             ON CONFLICT(account_id, thread_id) DO UPDATE SET \
             subject = excluded.subject, updated_at = excluded.updated_at",
        )
        .bind(account_id.to_string())
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
             FROM thread_subject_overrides WHERE account_id = ?",
        )
        .bind(account_id.to_string())
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, account_id: Uuid, thread_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM thread_subject_overrides WHERE account_id = ? AND thread_id = ?")
            .bind(account_id.to_string())
            .bind(thread_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
