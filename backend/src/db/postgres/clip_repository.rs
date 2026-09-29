use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::{Clip, ThreadNote};
use crate::core::repository::{ClipRepository, ThreadNoteRepository};

pub struct PostgresClipRepository {
    pool: PgPool,
}

impl PostgresClipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClipRepository for PostgresClipRepository {
    async fn create(&self, clip: &Clip) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO clips (id, user_id, account_id, message_id, snippet, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(clip.id.0)
        .bind(clip.user_id.0)
        .bind(clip.account_id.0)
        .bind(&clip.message_id)
        .bind(&clip.snippet)
        .bind(clip.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<Clip>, sqlx::Error> {
        sqlx::query_as::<_, Clip>(
            "SELECT id, user_id, account_id, message_id, snippet, created_at \
             FROM clips WHERE user_id = $1 ORDER BY created_at DESC LIMIT 200",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM clips WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
    }
}

pub struct PostgresThreadNoteRepository {
    pool: PgPool,
}

impl PostgresThreadNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ThreadNoteRepository for PostgresThreadNoteRepository {
    async fn upsert(
        &self,
        account_id: Uuid,
        thread_id: &str,
        note: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO thread_notes (account_id, thread_id, note, updated_at) \
             VALUES ($1, $2, $3, EXTRACT(EPOCH FROM NOW())::BIGINT) \
             ON CONFLICT (account_id, thread_id) DO UPDATE SET \
             note = EXCLUDED.note, updated_at = EXCLUDED.updated_at",
        )
        .bind(account_id)
        .bind(thread_id)
        .bind(note)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<ThreadNote>, sqlx::Error> {
        sqlx::query_as::<_, ThreadNote>(
            "SELECT account_id, thread_id, note, updated_at \
             FROM thread_notes WHERE account_id = $1",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, account_id: Uuid, thread_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM thread_notes WHERE account_id = $1 AND thread_id = $2")
            .bind(account_id)
            .bind(thread_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
