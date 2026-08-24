use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::models::Message;
use crate::core::repository::MessageRepository;

pub struct SqliteMessageRepository {
    pool: SqlitePool,
}

impl SqliteMessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

const MESSAGE_COLUMNS: &str = "m.id, m.account_id, m.external_id, m.thread_id, m.subject, m.sender_name, m.sender_email, \
     m.recipients, m.date_sent, m.date_received, m.snippet, m.body_text, m.body_html, m.labels, \
     m.is_read, m.is_archived, m.is_deleted, m.has_attachments, m.snoozed_until, m.has_conflict, m.created_at, m.updated_at";

#[async_trait]
impl MessageRepository for SqliteMessageRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.id = ?"
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(&format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m WHERE m.account_id = ? AND m.external_id = ?"
        ))
        .bind(account_id.to_string())
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
        let mut conditions = vec!["m.is_deleted = 0".to_string()];
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(account_id) = account_id {
            conditions.push("m.account_id = ?".to_string());
            bind_values.push(account_id.to_string());
        }

        if let Some(folder) = folder {
            match folder {
                "archived" => conditions.push("m.is_archived = 1".to_string()),
                "inbox" => conditions.push("m.is_archived = 0".to_string()),
                _ => {
                    conditions.push("m.labels LIKE ?".to_string());
                    bind_values.push(format!("%{}%", folder));
                }
            }
        }

        if let Some(cursor) = cursor {
            conditions.push("m.date_received < ?".to_string());
            bind_values.push(cursor.to_string());
        }

        let where_clause = conditions.join(" AND ");
        let query = format!(
            "SELECT {MESSAGE_COLUMNS} FROM messages m \
             WHERE {where_clause} ORDER BY m.date_received DESC LIMIT ?"
        );

        let mut q = sqlx::query_as::<_, Message>(&query);
        for val in &bind_values {
            q = q.bind(val);
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
             JOIN messages_fts fts ON m.rowid = fts.rowid \
             WHERE a.user_id = ? AND messages_fts MATCH ? \
             ORDER BY m.date_received DESC LIMIT ?"
        ))
        .bind(user_id.to_string())
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    async fn upsert(&self, message: &Message) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO messages (id, account_id, external_id, thread_id, subject, sender_name, \
             sender_email, recipients, date_sent, date_received, snippet, body_text, body_html, \
             labels, is_read, is_archived, is_deleted, has_attachments, snoozed_until, has_conflict, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(account_id, external_id) DO UPDATE SET \
             thread_id = excluded.thread_id, subject = excluded.subject, \
             sender_name = excluded.sender_name, sender_email = excluded.sender_email, \
             recipients = excluded.recipients, date_sent = excluded.date_sent, \
             date_received = excluded.date_received, snippet = excluded.snippet, \
             body_text = excluded.body_text, body_html = excluded.body_html, \
             labels = excluded.labels, is_read = excluded.is_read, \
             is_archived = excluded.is_archived, is_deleted = excluded.is_deleted, \
             has_attachments = excluded.has_attachments, snoozed_until = excluded.snoozed_until, has_conflict = excluded.has_conflict, \
             updated_at = excluded.updated_at",
        )
        .bind(message.id.to_string())
        .bind(message.account_id.to_string())
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
        .bind(message.has_conflict)
        .bind(message.created_at)
        .bind(message.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn set_read(&self, id: Uuid, is_read: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET is_read = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(is_read)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_archived(&self, id: Uuid, is_archived: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET is_archived = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(is_archived)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_deleted(&self, id: Uuid, is_deleted: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET is_deleted = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(is_deleted)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_labels(&self, id: Uuid, labels: Option<String>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE messages SET labels = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(labels)
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn set_thread_muted(&self, thread_id: &str) -> Result<(), sqlx::Error> {
        // Find current labels for the thread to append 'Muted'
        // For simplicity in SQLite we will just force append "Muted" if not present
        // and set is_archived = true.
        sqlx::query(
            "UPDATE messages SET \
             labels = CASE \
                WHEN labels IS NULL OR labels = '' THEN 'Muted' \
                WHEN labels NOT LIKE '%Muted%' THEN labels || ',Muted' \
                ELSE labels \
             END, \
             is_archived = 1, \
             updated_at = unixepoch() \
             WHERE thread_id = ?",
        )
        .bind(thread_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn report_phishing(&self, id: Uuid) -> Result<(), sqlx::Error> {
        // Move to trash and add Phishing label
        sqlx::query(
            "UPDATE messages SET \
             is_deleted = 1, \
             labels = CASE \
                WHEN labels IS NULL OR labels = '' THEN 'Phishing' \
                WHEN labels NOT LIKE '%Phishing%' THEN labels || ',Phishing' \
                ELSE labels \
             END, \
             updated_at = unixepoch() \
             WHERE id = ?",
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn trash_by_sender(&self, user_id: Uuid, email: &str) -> Result<(), sqlx::Error> {
        // Find accounts for this user to ensure we only touch their messages
        // But since we don't have user_id on messages, we join with accounts
        sqlx::query(
            "UPDATE messages SET \
             is_deleted = 1, \
             updated_at = unixepoch() \
             WHERE sender_email = ? AND account_id IN (SELECT id FROM accounts WHERE user_id = ?)",
        )
        .bind(email)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
