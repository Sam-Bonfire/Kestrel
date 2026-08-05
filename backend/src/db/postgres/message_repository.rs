use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::models::Message;
use crate::core::repository::MessageRepository;

pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const MESSAGE_COLUMNS: &str = "m.id, m.account_id, m.external_id, m.thread_id, m.subject, m.sender_name, m.sender_email, \
     m.recipients::TEXT as recipients, m.date_sent, m.date_received, m.snippet, m.body_text, m.body_html, \
     m.labels::TEXT as labels, m.is_read, m.is_archived, m.is_deleted, m.has_attachments, m.snoozed_until, \
     m.created_at, m.updated_at";

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account_id = $1 AND m.external_id = $2"
        ))
        .bind(account_id)
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list(
        &self,
        account_id: Option<Uuid>,
        folder: Option<&str>,
        cursor: Option<&str>,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let limit = limit.min(50);
        let mut conditions = vec!["m.is_deleted = FALSE".to_string()];
        let mut param_idx = 1;

        if account_id.is_some() {
            conditions.push(format!("m.account_id = ${}", param_idx));
            param_idx += 1;
        }

        let mut folder_is_label = false;
        if let Some(folder) = folder {
            match folder {
                "archived" => conditions.push("m.is_archived = TRUE".to_string()),
                "inbox" => conditions.push("m.is_archived = FALSE".to_string()),
                _ => {
                    conditions.push(format!("m.labels::TEXT LIKE ${}", param_idx));
                    param_idx += 1;
                    folder_is_label = true;
                }
            }
        }

        if cursor.is_some() {
            conditions.push(format!("m.date_received < ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = conditions.join(" AND ");
        let query = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m \
             WHERE {where_clause} ORDER BY m.date_received DESC LIMIT ${}",
            param_idx
        );

        let mut q = sqlx::query_as::<_, Message>(&query);
        if let Some(aid) = account_id {
            q = q.bind(aid);
        }
        if let (Some(f), true) = (folder, folder_is_label) {
            q = q.bind(format!("%{}%", f));
        }
        if let Some(c) = cursor {
            q = q.bind(c);
        }
        q = q.bind(limit);
        q.fetch_all(&self.pool).await
    }

    async fn search(
        &self,
        user_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} \
             FROM messages m \
             JOIN accounts a ON m.account_id = a.id \
             WHERE a.user_id = $1 \
             AND (m.subject ILIKE '%' || $2 || '%' \
                  OR m.sender_name ILIKE '%' || $2 || '%' \
                  OR m.sender_email ILIKE '%' || $2 || '%' \
                  OR m.snippet ILIKE '%' || $2 || '%') \
             ORDER BY m.date_received DESC LIMIT $3"
        ))
        .bind(user_id)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, message: &Message) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO messages (id, account_id, external_id, thread_id, subject, sender_name, \
             sender_email, recipients, date_sent, date_received, snippet, body_text, body_html, \
             labels, is_read, is_archived, is_deleted, has_attachments, snoozed_until, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21) \
             ON CONFLICT (account_id, external_id) DO UPDATE SET \
             thread_id = EXCLUDED.thread_id, subject = EXCLUDED.subject, \
             sender_name = EXCLUDED.sender_name, sender_email = EXCLUDED.sender_email, \
             recipients = EXCLUDED.recipients, date_sent = EXCLUDED.date_sent, \
             date_received = EXCLUDED.date_received, snippet = EXCLUDED.snippet, \
             body_text = EXCLUDED.body_text, body_html = EXCLUDED.body_html, \
             labels = EXCLUDED.labels, is_read = EXCLUDED.is_read, \
             is_archived = EXCLUDED.is_archived, is_deleted = EXCLUDED.is_deleted, \
             has_attachments = EXCLUDED.has_attachments, snoozed_until = EXCLUDED.snoozed_until, \
             updated_at = EXCLUDED.updated_at",
        )
        .bind(message.id)
        .bind(message.account_id)
        .bind(&message.external_id)
        .bind(&message.thread_id)
        .bind(&message.subject)
        .bind(&message.sender_name)
        .bind(&message.sender_email)
        .bind(&message.recipients)
        .bind(message.date_sent)
        .bind(message.date_received)
        .bind(&message.snippet)
        .bind(&message.body_text)
        .bind(&message.body_html)
        .bind(&message.labels)
        .bind(message.is_read)
        .bind(message.is_archived)
        .bind(message.is_deleted)
        .bind(message.has_attachments)
        .bind(message.snoozed_until)
        .bind(message.created_at)
        .bind(message.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_read(&self, id: Uuid, is_read: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET is_read = $1, updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = $2",
        )
        .bind(is_read)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_archived(&self, id: Uuid, is_archived: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET is_archived = $1, updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = $2",
        )
        .bind(is_archived)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_deleted(&self, id: Uuid, is_deleted: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET is_deleted = $1, updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = $2",
        )
        .bind(is_deleted)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_labels(&self, id: Uuid, labels: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET labels = $1::JSONB, updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = $2",
        )
        .bind(labels)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_by_thread(
        &self,
        thread_id: &str,
        limit: i64,
    ) -> Result<Vec<Message>, sqlx::Error> {
        let limit = limit.min(50);
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m \
             WHERE m.thread_id = $1 ORDER BY m.date_received ASC LIMIT $2"
        ))
        .bind(thread_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn set_thread_muted(&self, thread_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET \
             labels = (CASE WHEN labels IS NULL THEN '[]'::jsonb ELSE labels::jsonb END) || '\"Muted\"'::jsonb, \
             is_archived = true, \
             updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT \
             WHERE thread_id = $1"
        )
        .bind(thread_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn report_phishing(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET \
             is_deleted = true, \
             labels = (CASE WHEN labels IS NULL THEN '[]'::jsonb ELSE labels::jsonb END) || '\"Phishing\"'::jsonb, \
             updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT \
             WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn trash_by_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE messages SET \
             is_deleted = true, \
             updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT \
             WHERE sender_email = $1 AND account_id IN (SELECT id FROM accounts WHERE user_id = $2)"
        )
        .bind(email)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
