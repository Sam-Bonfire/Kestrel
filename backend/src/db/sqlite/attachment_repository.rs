use crate::core::error::KestrelError;
use crate::core::models::Attachment;
use crate::core::types::DbUuid;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AttachmentRepository {
    pool: Arc<SqlitePool>,
}

impl AttachmentRepository {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }

    pub async fn get_attachments_for_message(
        &self,
        message_id: DbUuid,
    ) -> Result<Vec<Attachment>, KestrelError> {
        let attachments =
            sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE message_id = ?")
                .bind(message_id)
                .fetch_all(&*self.pool)
                .await
                .map_err(KestrelError::Database)?;

        Ok(attachments)
    }
}
