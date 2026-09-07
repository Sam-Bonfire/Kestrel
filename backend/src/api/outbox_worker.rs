use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::api::messages::SendMessageRequest;
use crate::core::models::{Message, OfflineQueueItem};
use crate::core::repository::{AccountRepository, MessageRepository};
use crate::core::types::DbUuid;
use crate::db::pool::DbPool;
use crate::plugins::manager::PluginManager;

/// Maximum delivery attempts before a queue row is dropped with a warning.
pub const MAX_RETRIES: i32 = 8;
const BASE_DELAY_SECS: i64 = 60;
const MAX_DELAY_SECS: i64 = 3600;

/// Exponential backoff in seconds, capped: 60, 120, 240, … 3600.
pub fn backoff_delay_secs(retry_count: i32) -> i64 {
    (BASE_DELAY_SECS << retry_count.min(10)).min(MAX_DELAY_SECS)
}

pub fn start_outbox_worker(db: DbPool, plugins: Arc<RwLock<PluginManager>>, jwt_secret: String) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = process_outbox_once(&db, &plugins, &jwt_secret).await {
                tracing::error!("Outbox worker encountered an error: {}", e);
            }
            sleep(Duration::from_secs(60)).await;
        }
    });
}

/// Run a single replay pass over due rows. Returns replayed count.
pub async fn process_outbox_once(
    db: &DbPool,
    plugins: &Arc<RwLock<PluginManager>>,
    jwt_secret: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let now = chrono::Utc::now().timestamp();
    let items: Vec<OfflineQueueItem> = match db {
        DbPool::Sqlite(pool) => {
            sqlx::query_as::<_, OfflineQueueItem>(
                "SELECT * FROM offline_queue WHERE retry_count < ? AND next_attempt_at <= ? \
                 ORDER BY queued_at ASC LIMIT 50",
            )
            .bind(MAX_RETRIES)
            .bind(now)
            .fetch_all(pool)
            .await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_as::<_, OfflineQueueItem>(
                "SELECT * FROM offline_queue WHERE retry_count < $1 AND next_attempt_at <= $2 \
                 ORDER BY queued_at ASC LIMIT 50",
            )
            .bind(MAX_RETRIES)
            .bind(now)
            .fetch_all(pool)
            .await?
        }
    };

    let mut replayed = 0;
    for item in items {
        match replay_item(db, plugins, jwt_secret, &item, now).await {
            Ok(true) => replayed += 1,
            Ok(false) => {}
            Err(e) => tracing::error!("Outbox replay of row {} failed: {}", item.id, e),
        }
    }
    Ok(replayed)
}

/// Returns true when the row was delivered and removed.
async fn replay_item(
    db: &DbPool,
    plugins: &Arc<RwLock<PluginManager>>,
    jwt_secret: &str,
    item: &OfflineQueueItem,
    now: i64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if item.action != "send-message" {
        tracing::warn!(
            "Dropping outbox row {} with unknown action '{}'",
            item.id,
            item.action
        );
        delete_row(db, item.id).await?;
        return Ok(false);
    }

    let request: SendMessageRequest = match item
        .payload
        .as_deref()
        .unwrap_or_default()
        .parse::<serde_json::Value>()
        .ok()
        .and_then(|v| serde_json::from_value(v).ok())
    {
        Some(req) => req,
        None => {
            tracing::warn!("Dropping outbox row {} with unparsable payload", item.id);
            delete_row(db, item.id).await?;
            return Ok(false);
        }
    };

    let account = match db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                jwt_secret.to_string(),
            )
            .find_by_id(request.account_id)
            .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                jwt_secret.to_string(),
            )
            .find_by_id(request.account_id)
            .await?
        }
    };
    let account = match account {
        Some(a) => a,
        None => {
            tracing::warn!("Dropping outbox row {}: account gone", item.id);
            delete_row(db, item.id).await?;
            return Ok(false);
        }
    };

    let guard = plugins.read().await;
    let Some(plugin) = guard.find_by_id(&account.provider) else {
        return bump_retry(db, item, now).await.map(|_| false);
    };

    let auth_token = match &account.access_token {
        Some(t) => t.clone(),
        None => {
            tracing::warn!("Dropping outbox row {}: account has no token", item.id);
            delete_row(db, item.id).await?;
            return Ok(false);
        }
    };

    let mapped_attachments = match map_attachments(&request) {
        Ok(mapped) => mapped,
        Err(e) => {
            tracing::warn!("Dropping outbox row {}: {}", item.id, e);
            delete_row(db, item.id).await?;
            return Ok(false);
        }
    };

    let send_result = plugin
        .as_mail_provider()
        .send_message(
            &auth_token,
            crate::plugins::traits::SendMessagePayload {
                to: request.to.clone(),
                cc: request.cc.clone(),
                bcc: request.bcc.clone(),
                subject: request.subject.clone(),
                body_html: request.body_html.clone().unwrap_or_default(),
                attachments: mapped_attachments,
            },
        )
        .await;

    match send_result {
        Ok(()) => {
            insert_sent_copy(db, &account.display_name, &account.id.0, &request).await?;
            delete_row(db, item.id).await?;
            Ok(true)
        }
        Err(e) => {
            tracing::warn!("Outbox row {} redelivery failed: {}", item.id, e);
            bump_retry(db, item, now).await.map(|_| false)
        }
    }
}

fn map_attachments(
    request: &SendMessageRequest,
) -> Result<Option<Vec<crate::plugins::traits::AttachmentPayload>>, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
    match request.attachments.as_ref() {
        None => Ok(None),
        Some(atts) => {
            let mut mapped = Vec::with_capacity(atts.len());
            for a in atts {
                let b64_str = if a.base64_content.contains(",") {
                    a.base64_content
                        .split(",")
                        .nth(1)
                        .unwrap_or(&a.base64_content)
                } else {
                    &a.base64_content
                };
                let bytes = b64
                    .decode(b64_str)
                    .map_err(|e| format!("Invalid base64 attachment: {e}"))?;
                mapped.push(crate::plugins::traits::AttachmentPayload {
                    filename: a.filename.clone(),
                    content_type: a.content_type.clone(),
                    content: bytes,
                });
            }
            Ok(Some(mapped))
        }
    }
}

/// Mirror of the local copy written by the direct send path.
async fn insert_sent_copy(
    db: &DbPool,
    sender_display: &str,
    account_id: &uuid::Uuid,
    request: &SendMessageRequest,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().timestamp();
    let msg = Message {
        id: DbUuid(id),
        account_id: DbUuid(*account_id),
        external_id: format!("local-sent-{id}"),
        thread_id: format!("thread-{id}"),
        subject: Some(request.subject.clone()),
        sender_name: None,
        sender_email: sender_display.to_string(),
        recipients: request.to.join(", "),
        date_sent: now,
        date_received: now,
        snippet: Some("Sent message...".to_string()),
        body_text: request.body_text.clone(),
        body_html: request.body_html.clone(),
        labels: Some("[\"SENT\"]".to_string()),
        is_read: true,
        is_archived: false,
        is_deleted: false,
        has_attachments: request.attachments.is_some(),
        snoozed_until: None,
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    match db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone())
                .upsert(&msg)
                .await?;
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::message_repository::PostgresMessageRepository::new(pool.clone())
                .upsert(&msg)
                .await?;
        }
    }
    Ok(())
}

async fn delete_row(db: &DbPool, id: i32) -> Result<(), sqlx::Error> {
    match db {
        DbPool::Sqlite(pool) => {
            sqlx::query("DELETE FROM offline_queue WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("DELETE FROM offline_queue WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

async fn bump_retry(
    db: &DbPool,
    item: &OfflineQueueItem,
    now: i64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let next = item.retry_count + 1;
    if next >= MAX_RETRIES {
        tracing::warn!("Dropping outbox row {} after {} attempts", item.id, next);
        delete_row(db, item.id).await?;
        return Ok(());
    }
    let next_attempt = now + backoff_delay_secs(next);
    match db {
        DbPool::Sqlite(pool) => {
            sqlx::query(
                "UPDATE offline_queue SET retry_count = ?, next_attempt_at = ? WHERE id = ?",
            )
            .bind(next)
            .bind(next_attempt)
            .bind(item.id)
            .execute(pool)
            .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(
                "UPDATE offline_queue SET retry_count = $1, next_attempt_at = $2 WHERE id = $3",
            )
            .bind(next)
            .bind(next_attempt)
            .bind(item.id)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_grows_exponentially_and_caps() {
        assert_eq!(backoff_delay_secs(0), 60);
        assert_eq!(backoff_delay_secs(1), 120);
        assert_eq!(backoff_delay_secs(2), 240);
        assert_eq!(backoff_delay_secs(6), 3600);
        assert_eq!(backoff_delay_secs(100), 3600);
    }
}
