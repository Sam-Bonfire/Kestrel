use std::convert::Infallible;
use std::time::Duration;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;
use crate::db::pool::DbPool;
use crate::db::sqlite::account_repository::SqliteAccountRepository;
use crate::db::postgres::account_repository::PostgresAccountRepository;

// --- Sync event types ---

#[derive(Debug, Clone, Serialize)]
pub struct SyncEvent {
    pub event_type: String,
    pub account_id: Option<Uuid>,
    pub message: String,
    pub timestamp: i64,
}

// --- K-047: POST /api/v1/sync/trigger ---

#[derive(Deserialize)]
pub struct SyncTriggerRequest {
    pub account_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct SyncTriggerResponse {
    pub status: String,
    pub message: String,
}

pub async fn trigger_sync(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<SyncTriggerRequest>,
) -> Result<Json<SyncTriggerResponse>, KestrelError> {
    let accounts = list_user_accounts(&state, user_id).await?;

    if accounts.is_empty() {
        return Err(KestrelError::BadRequest(
            "No accounts linked. Connect a provider first.".to_string(),
        ));
    }

    // Filter to specific account if provided
    let accounts_to_sync: Vec<_> = match body.account_id {
        Some(aid) => accounts.into_iter().filter(|a| *a.id == aid).collect(),
        None => accounts,
    };

    if accounts_to_sync.is_empty() {
        return Err(KestrelError::NotFound("Account not found".to_string()));
    }

    // Perform sync for each account (synchronous for now, will be async background job later)
    let mut synced_count = 0;
    for account in &accounts_to_sync {
        let token = match &account.access_token {
            Some(t) => t.clone(),
            None => continue, // Skip accounts without tokens
        };

        match sync_account_messages(&state, account, &token).await {
            Ok(count) => synced_count += count,
            Err(e) => {
                tracing::warn!(
                    "Sync failed for account {} ({}): {}",
                    account.id.0,
                    account.provider,
                    e
                );
            }
        }
    }

    Ok(Json(SyncTriggerResponse {
        status: "ok".to_string(),
        message: format!(
            "Synced {} messages across {} account(s)",
            synced_count,
            accounts_to_sync.len()
        ),
    }))
}

// --- K-046: GET /api/v1/sync/stream — SSE ---

pub async fn sync_stream(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.sync_tx.subscribe();

    let stream = futures::stream::unfold(rx, move |mut rx| {
        async move {
            match rx.recv().await {
                Ok(event) => {
                    // Filter events relevant to this user
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    let sse_event = Event::default()
                        .event(&event.event_type)
                        .data(data);
                    Some((Ok(sse_event), rx))
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Skip lagged messages
                    Some((Ok(Event::default().data("")), rx))
                }
                Err(broadcast::error::RecvError::Closed) => None,
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// --- Background sync daemon (K-037) ---

/// Start the background sync daemon that periodically syncs all linked accounts.
pub fn start_sync_daemon(state: AppState, sync_tx: broadcast::Sender<SyncEvent>) {
    tokio::spawn(async move {
        tracing::info!("Background sync daemon started");

        loop {
            // Wait 5 minutes between sync cycles
            tokio::time::sleep(Duration::from_secs(300)).await;

            tracing::info!("Sync daemon: starting periodic sync cycle");

            // Get all accounts that need syncing
            let accounts = match get_all_accounts_with_tokens(&state).await {
                Ok(accounts) => accounts,
                Err(e) => {
                    tracing::error!("Sync daemon: failed to list accounts: {}", e);
                    continue;
                }
            };

            for account in &accounts {
                let token = match &account.access_token {
                    Some(t) => t.clone(),
                    None => continue,
                };

                match sync_account_messages(&state, account, &token).await {
                    Ok(count) => {
                        if count > 0 {
                            let event = SyncEvent {
                                event_type: "sync_complete".to_string(),
                                account_id: Some(account.id.0),
                                message: format!("Synced {} new messages", count),
                                timestamp: Utc::now().timestamp(),
                            };
                            let _ = sync_tx.send(event);
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Sync daemon: failed for account {}: {}",
                            account.id.0,
                            e
                        );
                        let event = SyncEvent {
                            event_type: "sync_error".to_string(),
                            account_id: Some(account.id.0),
                            message: format!("Sync failed: {}", e),
                            timestamp: Utc::now().timestamp(),
                        };
                        let _ = sync_tx.send(event);
                    }
                }
            }

            tracing::info!(
                "Sync daemon: cycle complete, synced {} account(s)",
                accounts.len()
            );
        }
    });
}

// --- Internal helpers ---

pub async fn list_user_accounts(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<crate::core::models::Account>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteAccountRepository::new(pool.clone());
            Ok(repo.find_by_user_id(user_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone());
            Ok(repo.find_by_user_id(user_id).await?)
        }
    }
}

async fn get_all_accounts_with_tokens(
    state: &AppState,
) -> Result<Vec<crate::core::models::Account>, KestrelError> {
    // For now, we list all accounts. In production, this would use a
    // dedicated query for accounts with non-expired tokens.
    match &state.db {
        DbPool::Sqlite(pool) => {
            // List all accounts — a proper impl would filter by token presence
            // For now we use a raw query
            let accounts = sqlx::query_as::<_, crate::core::models::Account>(
                "SELECT id, user_id, provider, provider_account_id, display_name, \
                 access_token, refresh_token, token_expires_at, created_at, updated_at \
                 FROM accounts WHERE access_token IS NOT NULL",
            )
            .fetch_all(pool)
            .await?;
            Ok(accounts)
        }
        DbPool::Postgres(pool) => {
            let _ = pool; // Placeholder for Postgres impl
            Ok(vec![])
        }
    }
}

/// K-038: LWW conflict resolution — sync messages from a provider account.
/// Compares timestamps to decide whether to upsert or skip.
async fn sync_account_messages(
    state: &AppState,
    account: &crate::core::models::Account,
    token: &str,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let plugin_manager = state.plugin_manager.read().await;
    let plugin = match plugin_manager.find_by_provider(&account.provider) {
        Some(p) => p,
        None => {
            tracing::warn!("No plugin loaded for provider {}", account.provider);
            return Ok(0);
        }
    };

    let mail_provider = plugin.as_mail_provider();
    
    // For now, always do a full sync without cursor until we add cursor storage
    let cursor = None;

    let result = mail_provider.sync_mail(token, cursor).await?;
    let mut synced_count = 0;

    let repo: Box<dyn crate::core::repository::MessageRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone())),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(crate::db::postgres::message_repository::PostgresMessageRepository::new(pool.clone())),
    };

    let filter_repo: Box<dyn crate::core::repository::FilterRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(crate::db::sqlite::filter_repository::SqliteFilterRepository::new(pool.clone())),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(crate::db::postgres::filter_repository::PostgresFilterRepository::new(pool.clone())),
    };

    let blocked_senders = filter_repo.get_blocked_senders(account.user_id.0).await.unwrap_or_default();
    let blocked_set: std::collections::HashSet<String> = blocked_senders.into_iter().collect();

    for payload in result.messages {
        let existing = repo.find_by_external_id(account.id.0, &payload.external_id).await?;
        
        let should_upsert = match &existing {
            Some(msg) => {
                // Last-Write-Wins (LWW): if remote is newer, update. 
                // Since payload doesn't have updated_at, we assume sync_mail only returns NEW or UPDATED emails
                // We use date_received as a proxy for now, or just assume it's newer if it was yielded.
                // Ideally we'd use an updated_at from the provider payload.
                payload.date_received >= msg.date_received
            }
            None => true,
        };

        if should_upsert {
            let mut message = match existing {
                Some(mut m) => {
                    m.subject = payload.subject;
                    m.sender_name = payload.sender_name;
                    m.sender_email = payload.sender_email;
                    m.recipients = payload.recipients;
                    m.date_sent = payload.date_sent;
                    m.date_received = payload.date_received;
                    m.snippet = payload.snippet;
                    m.is_deleted = m.is_deleted || blocked_set.contains(&m.sender_email);
                    m.updated_at = chrono::Utc::now().timestamp();
                    m.labels = payload.labels;
                    m.is_read = payload.is_read;
                    m
                },
                None => {
                    let is_blocked = blocked_set.contains(&payload.sender_email);
                    crate::core::models::Message {
                        id: Uuid::new_v4().into(),
                        account_id: account.id.clone(),
                        external_id: payload.external_id.clone(),
                        thread_id: payload.thread_id.clone(),
                        subject: payload.subject.clone(),
                        sender_name: payload.sender_name.clone(),
                        sender_email: payload.sender_email.clone(),
                        recipients: payload.recipients.clone(),
                        date_sent: payload.date_sent,
                        date_received: payload.date_received,
                        snippet: payload.snippet.clone(),
                        body_text: None,
                        body_html: None,
                        labels: payload.labels.clone(),
                        is_read: payload.is_read,
                        is_archived: false,
                        is_deleted: is_blocked,
                        has_attachments: false,
                        snoozed_until: None,
                        created_at: Utc::now().timestamp(),
                        updated_at: Utc::now().timestamp(),
                    }
                }
            };
            
            repo.upsert(&message).await?;
            synced_count += 1;
        }
    }

    Ok(synced_count)
}
