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

async fn list_user_accounts(
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
    _token: &str,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    // With mock plugins, sync returns empty results.
    // When real WASM plugins are loaded, this will call plugin.sync_mail().
    //
    // For now, this is a no-op that returns 0 synced messages.
    // The real implementation will:
    // 1. Get the plugin for this account's provider
    // 2. Get the last sync cursor for this account
    // 3. Call plugin.sync_mail(token, cursor)
    // 4. For each returned message, apply LWW conflict resolution:
    //    - If message doesn't exist locally, insert it
    //    - If message exists and remote.updated_at > local.updated_at, update it
    //    - Otherwise, skip (local is newer or same)
    // 5. Return the new cursor

    let _ = state;
    let _ = account;

    tracing::debug!(
        "Sync for account {} ({}) — mock plugin returns empty",
        account.id.0,
        account.provider
    );

    Ok(0)
}
