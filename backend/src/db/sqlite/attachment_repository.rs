use sqlx::SqlitePool;
use crate::core::error::KestrelError;
use crate::core::models::Attachment;
use crate::core::types::DbUuid;
use std::sync::Arc;

pub struct AttachmentRepository {
    pool: Arc<SqlitePool>,
}

impl AttachmentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn create_attachment(&self, attachment: &Attachment) -> Result<(), KestrelError> {
        sqlx::query(
            r#"
            INSERT INTO attachments (id, message_id, filename, content_type, size, external_id, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(attachment.id)
        .bind(attachment.message_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size)
        .bind(&attachment.external_id)
        .bind(attachment.created_at)
        .execute(&*self.pool)
        .await
        .map_err(KestrelError::Database)?;

        Ok(())
    }

    pub async fn get_attachments_for_message(&self, message_id: DbUuid) -> Result<Vec<Attachment>, KestrelError> {
        let attachments = sqlx::query_as::<_, Attachment>(
            "SELECT * FROM attachments WHERE message_id = ?"
        )
        .bind(message_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(KestrelError::Database)?;

        Ok(attachments)
    }
}
