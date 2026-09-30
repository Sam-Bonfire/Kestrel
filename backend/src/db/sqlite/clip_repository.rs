use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::{Clip, ThreadNote};
use crate::core::repository::{ClipRepository, ThreadNoteRepository};

pub struct SqliteClipRepository {
    pool: SqlitePool,
}

impl SqliteClipRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClipRepository for SqliteClipRepository {
    async fn create(&self, clip: &Clip) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO clips (id, user_id, account_id, message_id, snippet, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(clip.id.to_string())
        .bind(clip.user_id.to_string())
        .bind(clip.account_id.to_string())
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
             FROM clips WHERE user_id = ? ORDER BY created_at DESC, rowid DESC LIMIT 200",
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM clips WHERE id = ? AND user_id = ?")
            .bind(id.to_string())
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() == 1)
    }
}

pub struct SqliteThreadNoteRepository {
    pool: SqlitePool,
}

impl SqliteThreadNoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ThreadNoteRepository for SqliteThreadNoteRepository {
    async fn upsert(
        &self,
        account_id: Uuid,
        thread_id: &str,
        note: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO thread_notes (account_id, thread_id, note, updated_at) \
             VALUES (?, ?, ?, unixepoch()) \
             ON CONFLICT(account_id, thread_id) DO UPDATE SET \
             note = excluded.note, updated_at = excluded.updated_at",
        )
        .bind(account_id.to_string())
        .bind(thread_id)
        .bind(note)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_account(&self, account_id: Uuid) -> Result<Vec<ThreadNote>, sqlx::Error> {
        sqlx::query_as::<_, ThreadNote>(
            "SELECT account_id, thread_id, note, updated_at \
             FROM thread_notes WHERE account_id = ?",
        )
        .bind(account_id.to_string())
        .fetch_all(&self.pool)
        .await
    }

    async fn delete(&self, account_id: Uuid, thread_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM thread_notes WHERE account_id = ? AND thread_id = ?")
            .bind(account_id.to_string())
            .bind(thread_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
