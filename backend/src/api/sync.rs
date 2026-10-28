use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{Semaphore, broadcast};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::HistoricalRevision;
use crate::core::repository::{AccountRepository, HistoricalRevisionRepository};
use crate::db::pool::DbPool;
use crate::db::postgres::account_repository::PostgresAccountRepository;
use crate::db::sqlite::account_repository::SqliteAccountRepository;

// --- Sync event types ---

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event_type")]
pub enum SyncEvent {
    #[serde(rename = "sync_started")]
    SyncStarted {
        account_id: String,
        provider: String,
    },
    #[serde(rename = "sync_progress")]
    SyncProgress {
        account_id: String,
        stage: String,
        items_synced: u32,
        total_items: Option<u32>,
    },
    #[serde(rename = "sync_complete")]
    SyncComplete {
        account_id: String,
        duration_ms: u64,
    },
    #[serde(rename = "sync_error")]
    SyncError { account_id: String, error: String },
    #[serde(rename = "auth_revocation")]
    AuthRevocation {
        account_id: String,
        provider: String,
        message: String,
        timestamp: i64,
    },
    #[serde(rename = "message_unsnoozed")]
    MessageUnsnoozed { message_id: String, timestamp: i64 },
}

// --- Token Refresher ---

#[async_trait::async_trait]
pub trait TokenRefresher: Send + Sync {
    async fn refresh(
        &self,
        account: &crate::core::models::Account,
    ) -> Result<serde_json::Value, String>;
}

pub struct ReqwestTokenRefresher {
    client: reqwest::Client,
}

impl Default for ReqwestTokenRefresher {
    fn default() -> Self {
        Self::new()
    }
}

impl ReqwestTokenRefresher {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl TokenRefresher for ReqwestTokenRefresher {
    async fn refresh(
        &self,
        account: &crate::core::models::Account,
    ) -> Result<serde_json::Value, String> {
        let (token_url, client_id, client_secret) = match account.provider.as_str() {
            "gmail" => (
                "https://oauth2.googleapis.com/token",
                std::env::var("GMAIL_CLIENT_ID")
                    .or_else(|_| std::env::var("GOOGLE_CLIENT_ID"))
                    .unwrap_or_default(),
                std::env::var("GMAIL_CLIENT_SECRET")
                    .or_else(|_| std::env::var("GOOGLE_CLIENT_SECRET"))
                    .unwrap_or_default(),
            ),
            "outlook" => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                std::env::var("OUTLOOK_CLIENT_ID")
                    .or_else(|_| std::env::var("MICROSOFT_CLIENT_ID"))
                    .unwrap_or_default(),
                std::env::var("OUTLOOK_CLIENT_SECRET")
                    .or_else(|_| std::env::var("MICROSOFT_CLIENT_SECRET"))
                    .unwrap_or_default(),
            ),
            _ => return Err("Unknown provider".to_string()),
        };

        if client_id.is_empty() || client_secret.is_empty() {
            return Err("Missing OAuth credentials in env".to_string());
        }

        let refresh_token = match &account.refresh_token {
            Some(rt) => rt.as_str(),
            None => return Err("No refresh token available".to_string()),
        };

        let res = self
            .client
            .post(token_url)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = res.status();
        let text = res.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(format!("Provider error ({}): {}", status, text));
        }

        serde_json::from_str(&text).map_err(|e| format!("Invalid JSON from provider: {}", e))
    }
}

pub async fn ensure_valid_token(
    state: &AppState,
    account: &mut crate::core::models::Account,
    refresher: &dyn TokenRefresher,
) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp();
    // 5 minutes = 300 seconds
    let threshold = now + 300;

    let needs_refresh = account.access_token.is_none()
        || account.token_expires_at.is_none()
        || account.token_expires_at.unwrap() < threshold;

    if !needs_refresh && let Some(ref token) = account.access_token {
        return Ok(token.clone());
    }

    match refresher.refresh(account).await {
        Ok(token_data) => {
            if let Some(access_token) = token_data["access_token"].as_str() {
                let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);
                account.access_token = Some(access_token.to_string());
                account.token_expires_at = Some(now + expires_in);
                account.sync_error = None;
                account.updated_at = chrono::Utc::now().timestamp();

                // Save back to database
                match &state.db {
                    DbPool::Sqlite(pool) => {
                        if let Err(e) =
                            SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                .update_tokens_and_error(
                                    account.id.0,
                                    Some(access_token),
                                    account.token_expires_at,
                                    None,
                                )
                                .await
                        {
                            tracing::error!("DB update failed: {}", e);
                        }
                    }
                    DbPool::Postgres(pool) => {
                        if let Err(e) =
                            PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                .update_tokens_and_error(
                                    account.id.0,
                                    Some(access_token),
                                    account.token_expires_at,
                                    None,
                                )
                                .await
                        {
                            tracing::error!("DB update failed: {}", e);
                        }
                    }
                }
                Ok(access_token.to_string())
            } else {
                let err_msg = "Invalid token payload received from provider".to_string();
                account.sync_error = Some(err_msg.clone());
                match &state.db {
                    DbPool::Sqlite(pool) => {
                        if let Err(e) =
                            SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                .update_tokens_and_error(
                                    account.id.0,
                                    account.access_token.as_deref(),
                                    account.token_expires_at,
                                    Some(&err_msg),
                                )
                                .await
                        {
                            tracing::error!("DB update failed: {}", e);
                        }
                    }
                    DbPool::Postgres(pool) => {
                        if let Err(e) =
                            PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                .update_tokens_and_error(
                                    account.id.0,
                                    account.access_token.as_deref(),
                                    account.token_expires_at,
                                    Some(&err_msg),
                                )
                                .await
                        {
                            tracing::error!("DB update failed: {}", e);
                        }
                    }
                }
                let event = SyncEvent::AuthRevocation {
                    account_id: account.id.0.to_string(),
                    provider: account.provider.clone(),
                    message: "Token refresh permanently failed (e.g. revoked)".to_string(),
                    timestamp: chrono::Utc::now().timestamp(),
                };
                let _ = state.sync_tx.send(event);
                Err(err_msg)
            }
        }
        Err(e) => {
            account.sync_error = Some(e.clone());
            let event = SyncEvent::AuthRevocation {
                account_id: account.id.0.to_string(),
                provider: account.provider.clone(),
                message: "Token refresh permanently failed (e.g. revoked)".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = state.sync_tx.send(event);
            match &state.db {
                DbPool::Sqlite(pool) => {
                    if let Err(e) =
                        SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                            .update_tokens_and_error(
                                account.id.0,
                                account.access_token.as_deref(),
                                account.token_expires_at,
                                Some(&e),
                            )
                            .await
                    {
                        tracing::error!("DB update failed: {}", e);
                    }
                }
                DbPool::Postgres(pool) => {
                    if let Err(e) =
                        PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                            .update_tokens_and_error(
                                account.id.0,
                                account.access_token.as_deref(),
                                account.token_expires_at,
                                Some(&e),
                            )
                            .await
                    {
                        tracing::error!("DB update failed: {}", e);
                    }
                }
            }
            Err(e)
        }
    }
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
    let len = accounts_to_sync.len();
    for mut account in accounts_to_sync {
        let refresher = ReqwestTokenRefresher::new();
        let token = match ensure_valid_token(&state, &mut account, &refresher).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(
                    "Sync trigger token refresh failed for {}: {}",
                    account.id.0,
                    e
                );
                continue;
            }
        };

        match sync_account_messages(&state, &account, &token, &state.sync_tx).await {
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
        message: format!("Synced {} messages across {} account(s)", synced_count, len),
    }))
}

// --- K-046: GET /api/v1/sync/stream — SSE ---

fn event_account_id(event: &SyncEvent) -> Option<&str> {
    match event {
        SyncEvent::SyncStarted { account_id, .. }
        | SyncEvent::SyncProgress { account_id, .. }
        | SyncEvent::SyncComplete { account_id, .. }
        | SyncEvent::SyncError { account_id, .. }
        | SyncEvent::AuthRevocation { account_id, .. } => Some(account_id),
        SyncEvent::MessageUnsnoozed { .. } => None,
    }
}

async fn user_account_ids(
    state: &AppState,
    user_id: Uuid,
) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let ids = sqlx::query_scalar::<_, String>("SELECT id FROM accounts WHERE user_id = ?")
                .bind(user_id.to_string())
                .fetch_all(pool)
                .await?;
            Ok(ids.into_iter().collect())
        }
        DbPool::Postgres(pool) => {
            let ids = sqlx::query_scalar::<_, Uuid>("SELECT id FROM accounts WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(pool)
                .await?;
            Ok(ids.into_iter().map(|id| id.to_string()).collect())
        }
    }
}

async fn message_belongs_to_user(
    state: &AppState,
    user_id: Uuid,
    message_id: &str,
) -> Result<bool, sqlx::Error> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM messages m JOIN accounts a ON m.account_id = a.id \
                 WHERE m.id = ? AND a.user_id = ?",
            )
            .bind(message_id)
            .bind(user_id.to_string())
            .fetch_one(pool)
            .await?;
            Ok(count > 0)
        }
        DbPool::Postgres(pool) => {
            let Ok(message_id) = Uuid::parse_str(message_id) else {
                return Ok(false);
            };
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM messages m JOIN accounts a ON m.account_id = a.id \
                 WHERE m.id = $1 AND a.user_id = $2",
            )
            .bind(message_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
            Ok(count > 0)
        }
    }
}

async fn event_is_visible(
    state: &AppState,
    user_id: Uuid,
    account_ids: &std::collections::HashSet<String>,
    event: &SyncEvent,
) -> bool {
    match event_account_id(event) {
        Some(account_id) => account_ids.contains(account_id),
        None => match event {
            SyncEvent::MessageUnsnoozed { message_id, .. } => {
                message_belongs_to_user(state, user_id, message_id)
                    .await
                    .unwrap_or(false)
            }
            _ => false,
        },
    }
}

pub async fn sync_stream(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.sync_tx.subscribe();
    let account_ids = user_account_ids(&state, user_id).await.unwrap_or_default();

    let stream = futures::stream::unfold(
        (rx, state, account_ids, user_id),
        |(mut rx, state, account_ids, user_id)| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if !event_is_visible(&state, user_id, &account_ids, &event).await {
                            continue;
                        }
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        let event_type = match &event {
                            SyncEvent::SyncStarted { .. } => "sync_started",
                            SyncEvent::SyncProgress { .. } => "sync_progress",
                            SyncEvent::SyncComplete { .. } => "sync_complete",
                            SyncEvent::SyncError { .. } => "sync_error",
                            SyncEvent::AuthRevocation { .. } => "auth_revocation",
                            SyncEvent::MessageUnsnoozed { .. } => "message_unsnoozed",
                        };
                        let sse_event = Event::default().event(event_type).data(data);
                        return Some((Ok(sse_event), (rx, state, account_ids, user_id)));
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        },
    );

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// --- Background sync daemon (K-037) ---

/// Start the background sync daemon that periodically syncs all linked accounts.
pub async fn run_sync_for_account(
    state: AppState,
    mut account: crate::core::models::Account,
    sync_tx: broadcast::Sender<SyncEvent>,
    semaphore: Option<Arc<Semaphore>>,
) {
    let refresher = ReqwestTokenRefresher::new();
    let token = match ensure_valid_token(&state, &mut account, &refresher).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(
                "Sync daemon token refresh failed for {}: {}",
                account.id.0,
                e
            );
            return;
        }
    };

    // Acquire per-provider concurrency permit if available
    let _permit = match semaphore {
        Some(sem) => Some(sem.acquire_owned().await.unwrap()),
        None => None,
    };

    let start_time = std::time::Instant::now();
    let _ = sync_tx.send(SyncEvent::SyncStarted {
        account_id: account.id.0.to_string(),
        provider: account.provider.clone(),
    });

    // 60-second timeout to prevent hanging on one provider
    let msg_sync = tokio::time::timeout(
        Duration::from_secs(60),
        sync_account_messages(&state, &account, &token, &sync_tx),
    )
    .await;

    match msg_sync {
        Ok(Ok(count)) => {
            tracing::info!("Synced {} messages for account {}", count, account.id.0);
        }
        Ok(Err(e)) => {
            tracing::warn!("Sync daemon failed for {}: {}", account.id.0, e);
            let _ = sync_tx.send(SyncEvent::SyncError {
                account_id: account.id.0.to_string(),
                error: e.to_string(),
            });
            return;
        }
        Err(_) => {
            tracing::warn!("Sync daemon timed out for {}", account.id.0);
            let _ = sync_tx.send(SyncEvent::SyncError {
                account_id: account.id.0.to_string(),
                error: "Sync timed out".to_string(),
            });
            return;
        }
    }

    let cal_sync = tokio::time::timeout(
        Duration::from_secs(60),
        sync_account_calendars(&state, &account, &token, &sync_tx),
    )
    .await;

    match cal_sync {
        Ok(Ok(count)) => {
            tracing::info!(
                "Synced {} new calendar events for account {}",
                count,
                account.id.0
            );
        }
        Ok(Err(e)) => {
            tracing::warn!("Calendar sync failed for {}: {}", account.id.0, e);
            let _ = sync_tx.send(SyncEvent::SyncError {
                account_id: account.id.0.to_string(),
                error: e.to_string(),
            });
            return;
        }
        Err(_) => {
            tracing::warn!("Calendar sync timed out for {}", account.id.0);
            let _ = sync_tx.send(SyncEvent::SyncError {
                account_id: account.id.0.to_string(),
                error: "Calendar sync timed out".to_string(),
            });
            return;
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let _ = sync_tx.send(SyncEvent::SyncComplete {
        account_id: account.id.0.to_string(),
        duration_ms,
    });
}

pub fn start_sync_daemon(
    state: AppState,
    sync_tx: broadcast::Sender<SyncEvent>,
    mut push_rx: tokio::sync::mpsc::Receiver<uuid::Uuid>,
) {
    tokio::spawn(async move {
        tracing::info!("Background sync daemon started");

        // Provider rate limiters (Max 5 concurrent requests per provider globally)
        let mut limits = HashMap::new();
        limits.insert("gmail".to_string(), Arc::new(Semaphore::new(5)));
        limits.insert("outlook".to_string(), Arc::new(Semaphore::new(5)));

        // Deduplication and Rate Limiting state
        let mut last_trigger_time: HashMap<uuid::Uuid, i64> = HashMap::new();
        let mut last_sync_time: HashMap<uuid::Uuid, i64> = HashMap::new();
        let mut queued_syncs: std::collections::HashSet<uuid::Uuid> =
            std::collections::HashSet::new();

        // 60-second fallback/polling interval
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            let mut accounts_to_sync: Vec<crate::core::models::Account> = Vec::new();

            tokio::select! {
                maybe_account_id = push_rx.recv() => {
                    match maybe_account_id {
                        Some(account_id) => {
                            let now = Utc::now().timestamp();

                            // Deduplication: if triggered within last 30s, ignore
                            if last_trigger_time.get(&account_id).is_some_and(|&last| now - last < 30) {
                                tracing::debug!("Sync trigger for {} deduplicated", account_id);
                                continue;
                            }
                            last_trigger_time.insert(account_id, now);

                            // Rate limiting: 1 sync per minute (60s)
                            if last_sync_time.get(&account_id).is_some_and(|&last_sync| now - last_sync < 60) {
                                tracing::debug!("Sync trigger for {} rate limited, queuing", account_id);
                                queued_syncs.insert(account_id);
                                continue;
                            }

                            last_sync_time.insert(account_id, now);

                            // Load account from DB to sync
                            let db_account = match &state.db {
                                DbPool::Sqlite(pool) => {
                                    crate::core::repository::AccountRepository::find_by_id(
                                        &crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                            .with_fallback_key(state.vault_old_secret.clone()),
                                        account_id
                                    ).await
                                }
                                DbPool::Postgres(pool) => {
                                    crate::core::repository::AccountRepository::find_by_id(
                                        &crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                                            .with_fallback_key(state.vault_old_secret.clone()),
                                        account_id
                                    ).await
                                }
                            };

                            if let Ok(Some(account)) = db_account {
                                accounts_to_sync.push(account);
                            }
                        }
                        None => {
                            // Channel closed, terminate daemon
                            tracing::info!("Sync daemon: push channel closed, terminating");
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    let now = Utc::now().timestamp();
                    tracing::debug!("Sync daemon: interval tick");

                    // Process unsnoozes
                    match &state.db {
                        DbPool::Sqlite(pool) => {
                            let repo = crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone());
                            if let Ok(unsnoozed_ids) = crate::core::repository::MessageRepository::unsnooze_due_messages(&repo, now).await {
                                for msg_id in unsnoozed_ids {
                                    let _ = sync_tx.send(SyncEvent::MessageUnsnoozed {
                                        message_id: msg_id.to_string(),
                                        timestamp: now,
                                    });
                                }
                            }
                        }
                        DbPool::Postgres(pool) => {
                            let repo = crate::db::postgres::message_repository::PostgresMessageRepository::new(pool.clone());
                            if let Ok(unsnoozed_ids) = crate::core::repository::MessageRepository::unsnooze_due_messages(&repo, now).await {
                                for msg_id in unsnoozed_ids {
                                    let _ = sync_tx.send(SyncEvent::MessageUnsnoozed {
                                        message_id: msg_id.to_string(),
                                        timestamp: now,
                                    });
                                }
                            }
                        }
                    }

                    if let Ok(accounts) = get_all_accounts_with_tokens(&state).await {
                        for account in accounts {
                            let account_id = account.id.0;
                            let last_sync = last_sync_time.get(&account_id).copied().unwrap_or(0);

                            let mut should_sync = false;

                            if queued_syncs.contains(&account_id) {
                                // If it was queued and the 60s rate limit has passed
                                if now - last_sync >= 60 {
                                    should_sync = true;
                                    queued_syncs.remove(&account_id);
                                }
                            } else if now - last_sync >= 300 {
                                // Fallback polling: if it hasn't synced in 5 minutes (300s)
                                should_sync = true;
                            }

                            if should_sync {
                                last_sync_time.insert(account_id, now);
                                accounts_to_sync.push(account);
                            }
                        }
                    }
                }
            }

            if accounts_to_sync.is_empty() {
                continue;
            }

            let accounts_len = accounts_to_sync.len();

            tracing::info!(
                "Sync daemon: spawning background tasks for {} account(s)",
                accounts_len
            );

            // Spawn tasks in the background so the daemon loop doesn't block
            for account in accounts_to_sync {
                let state_clone = state.clone();
                let tx_clone = sync_tx.clone();
                let semaphore = limits.get(&account.provider).cloned();

                tokio::spawn(async move {
                    run_sync_for_account(state_clone, account, tx_clone, semaphore).await;
                });
            }
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
            let repo = SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                .with_fallback_key(state.vault_old_secret.clone());
            Ok(repo.find_by_user_id(user_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                .with_fallback_key(state.vault_old_secret.clone());
            Ok(repo.find_by_user_id(user_id).await?)
        }
    }
}

async fn get_all_accounts_with_tokens(
    state: &AppState,
) -> Result<Vec<crate::core::models::Account>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let mut accounts = sqlx::query_as::<_, crate::core::models::Account>(
                "SELECT id, user_id, provider, provider_account_id, display_name, \
                 access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
                 FROM accounts WHERE access_token IS NOT NULL",
            )
            .fetch_all(pool)
            .await?;
            for acc in accounts.iter_mut() {
                if let Some(ref at) = acc.access_token {
                    acc.access_token = crate::core::crypto::decrypt_with_fallback(
                        at,
                        &state.vault_secret,
                        state.vault_old_secret.as_deref(),
                    )
                    .ok();
                }
                if let Some(ref rt) = acc.refresh_token {
                    acc.refresh_token = crate::core::crypto::decrypt_with_fallback(
                        rt,
                        &state.vault_secret,
                        state.vault_old_secret.as_deref(),
                    )
                    .ok();
                }
            }
            Ok(accounts)
        }
        DbPool::Postgres(pool) => {
            let mut accounts = sqlx::query_as::<_, crate::core::models::Account>(
                "SELECT id, user_id, provider, provider_account_id, display_name, \
                 access_token, refresh_token, token_expires_at, sync_error, created_at, updated_at \
                 FROM accounts WHERE access_token IS NOT NULL",
            )
            .fetch_all(pool)
            .await?;
            for acc in accounts.iter_mut() {
                if let Some(ref at) = acc.access_token {
                    acc.access_token = crate::core::crypto::decrypt_with_fallback(
                        at,
                        &state.vault_secret,
                        state.vault_old_secret.as_deref(),
                    )
                    .ok();
                }
                if let Some(ref rt) = acc.refresh_token {
                    acc.refresh_token = crate::core::crypto::decrypt_with_fallback(
                        rt,
                        &state.vault_secret,
                        state.vault_old_secret.as_deref(),
                    )
                    .ok();
                }
            }
            Ok(accounts)
        }
    }
}

/// K-295: LWW rule extracted for unit testing (locks K-038 behavior).
/// Equal timestamps apply: provider payloads carry no updated_at, so a
/// re-yielded payload is treated as newer to avoid missing body/label edits.
fn should_apply_remote_message(local_date_received: i64, remote_date_received: i64) -> bool {
    remote_date_received >= local_date_received
}

/// A row counts as locally edited once anything bumped `updated_at` past its
/// creation time (every API mutation does).
fn is_locally_modified(created_at: i64, updated_at: i64) -> bool {
    updated_at > created_at
}

fn message_content_differs(
    existing: &crate::core::models::Message,
    remote: &crate::plugins::traits::MessagePayload,
) -> bool {
    existing.subject != remote.subject
        || existing.sender_name != remote.sender_name
        || existing.sender_email != remote.sender_email
        || existing.recipients != remote.recipients
        || existing.date_sent != remote.date_sent
        || existing.date_received != remote.date_received
        || existing.snippet != remote.snippet
        || existing.labels != remote.labels
        || existing.is_read != remote.is_read
}

fn event_content_differs(
    existing: &crate::core::models::CalendarEvent,
    remote: &crate::plugins::traits::EventPayload,
) -> bool {
    existing.title != remote.title
        || existing.description != remote.description
        || existing.location != remote.location
        || existing.start_time != remote.start_time
        || existing.end_time != remote.end_time
        || existing.is_all_day != remote.is_all_day
        || existing.recurrence_rules != remote.recurrence_rules
        || existing.organizer_email != remote.organizer_email
        || existing.organizer_name != remote.organizer_name
        || existing.attendees != remote.attendees
        || existing.status != remote.status
}

/// Keep the version the remote sync is about to overwrite so it can be
/// restored, and mark the live row as conflicted.
async fn snapshot_conflict<T: Serialize>(
    repo: &dyn HistoricalRevisionRepository,
    resource_type: &str,
    resource_id: Uuid,
    local: &T,
) {
    let payload = match serde_json::to_string(local) {
        Ok(payload) => payload,
        Err(err) => {
            tracing::warn!(%err, resource_type, "Could not serialize conflicting row");
            return;
        }
    };
    let revision_number = match repo
        .get_latest_revision_number(resource_type, resource_id)
        .await
    {
        Ok(latest) => latest + 1,
        Err(err) => {
            tracing::warn!(%err, resource_type, "Could not read latest revision number");
            return;
        }
    };
    let revision = HistoricalRevision {
        id: Uuid::new_v4().into(),
        resource_type: resource_type.to_string(),
        resource_id: resource_id.into(),
        serialized_payload: payload,
        revision_number,
        created_at: Utc::now().timestamp(),
    };
    if let Err(err) = repo.create(&revision).await {
        tracing::warn!(%err, resource_type, "Could not persist conflict revision");
    }
}

/// K-038: LWW conflict resolution — sync messages from a provider account.
pub async fn sync_account_messages(
    state: &AppState,
    account: &crate::core::models::Account,
    token: &str,
    sync_tx: &broadcast::Sender<SyncEvent>,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let _ = sync_tx.send(SyncEvent::SyncProgress {
        account_id: account.id.0.to_string(),
        stage: "messages".to_string(),
        items_synced: 0,
        total_items: None,
    });

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
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::message_repository::PostgresMessageRepository::new(pool.clone()),
        ),
    };

    let filter_repo: Box<dyn crate::core::repository::FilterRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::filter_repository::SqliteFilterRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::filter_repository::PostgresFilterRepository::new(pool.clone()),
        ),
    };

    let contact_repo: Box<dyn crate::core::repository::ContactRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::contact_repository::SqliteContactRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::contact_repository::PostgresContactRepository::new(pool.clone()),
        ),
    };

    let revision_repo: Box<dyn HistoricalRevisionRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::revision_repository::SqliteRevisionRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::revision_repository::PostgresRevisionRepository::new(pool.clone()),
        ),
    };

    let blocked_senders = filter_repo
        .get_blocked_senders(account.user_id.0)
        .await
        .unwrap_or_default();
    let blocked_set: std::collections::HashSet<String> = blocked_senders.into_iter().collect();

    let total_items = result.messages.len() as u32;

    for payload in result.messages {
        let existing = repo
            .find_by_external_id(account.id.0, &payload.external_id)
            .await?;

        let should_upsert = match &existing {
            Some(msg) => {
                // Last-Write-Wins (LWW): if remote is newer, update.
                // Since payload doesn't have updated_at, we assume sync_mail only returns NEW or UPDATED emails
                // We use date_received as a proxy for now, or just assume it's newer if it was yielded.
                // Ideally we'd use an updated_at from the provider payload.
                should_apply_remote_message(msg.date_received, payload.date_received)
            }
            None => true,
        };

        let conflict = should_upsert
            && existing.as_ref().is_some_and(|m| {
                !m.has_conflict
                    && is_locally_modified(m.created_at, m.updated_at)
                    && message_content_differs(m, &payload)
            });
        if conflict && let Some(local) = &existing {
            snapshot_conflict(revision_repo.as_ref(), "message", local.id.0, local).await;
        }

        if should_upsert {
            // Upsert contact for the sender
            let contact = crate::core::models::Contact {
                id: Uuid::new_v4().into(),
                account_id: account.id,
                name: payload.sender_name.clone(),
                email: payload.sender_email.clone(),
                avatar_url: None,
                notes: None,
                last_contacted_at: payload.date_received,
                created_at: chrono::Utc::now().timestamp(),
            };
            // Merged-away contacts stay merged: sync must not resurrect them.
            if !contact_repo
                .is_merged(account.id.0, &contact.email)
                .await
                .unwrap_or(false)
            {
                let _ = contact_repo.upsert(&contact).await;
            }

            // Upsert contacts for recipients (parsing the JSON string if necessary, assuming it's a JSON array of strings or comma-separated string)
            // Kestrel mail recipients are typically stored as a JSON string `["a@b.com", "c@d.com"]` or comma-separated
            let parsed_recipients: Vec<String> = serde_json::from_str(&payload.recipients)
                .unwrap_or_else(|_| {
                    payload
                        .recipients
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                });

            for rec in parsed_recipients {
                if contact_repo
                    .is_merged(account.id.0, &rec)
                    .await
                    .unwrap_or(false)
                {
                    continue;
                }
                let contact = crate::core::models::Contact {
                    id: Uuid::new_v4().into(),
                    account_id: account.id,
                    name: None,
                    email: rec,
                    avatar_url: None,
                    notes: None,
                    last_contacted_at: payload.date_sent, // fallback to date_sent
                    created_at: chrono::Utc::now().timestamp(),
                };
                let _ = contact_repo.upsert(&contact).await;
            }

            let message = match existing {
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
                    m.has_conflict = m.has_conflict || conflict;
                    m
                }
                None => {
                    let is_blocked = blocked_set.contains(&payload.sender_email);
                    crate::core::models::Message {
                        id: Uuid::new_v4().into(),
                        account_id: account.id,
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
                        is_set_aside: false,
                        has_conflict: false,
                        created_at: Utc::now().timestamp(),
                        updated_at: Utc::now().timestamp(),
                    }
                }
            };

            repo.upsert(&message).await?;
            synced_count += 1;

            if synced_count % 10 == 0 || synced_count == total_items as i64 {
                let _ = sync_tx.send(SyncEvent::SyncProgress {
                    account_id: account.id.0.to_string(),
                    stage: "messages".to_string(),
                    items_synced: synced_count as u32,
                    total_items: Some(total_items),
                });
            }
        }
    }

    Ok(synced_count)
}

/// Task 2.2: Fetch and store calendar events.
pub async fn sync_account_calendars(
    state: &AppState,
    account: &crate::core::models::Account,
    token: &str,
    sync_tx: &broadcast::Sender<SyncEvent>,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
    let _ = sync_tx.send(SyncEvent::SyncProgress {
        account_id: account.id.0.to_string(),
        stage: "calendars".to_string(),
        items_synced: 0,
        total_items: None,
    });

    let plugin_manager = state.plugin_manager.read().await;
    let plugin = match plugin_manager.find_by_provider(&account.provider) {
        Some(p) => p,
        None => return Ok(0),
    };

    let calendar_provider = plugin.as_calendar_provider();

    // Sync calendars (metadata) first
    let calendars = calendar_provider.fetch_calendars(token).await?;
    let mut default_calendar_id = None;

    let cal_repo: Box<dyn crate::core::repository::CalendarRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::calendar_repository::SqliteCalendarRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::calendar_repository::PostgresCalendarRepository::new(pool.clone()),
        ),
    };

    let calendars_for_account = cal_repo.list_by_account(account.id.0).await?;

    for payload in calendars {
        let existing = calendars_for_account
            .iter()
            .find(|c| c.external_id == payload.id)
            .cloned();
        let calendar_db_id = match existing {
            Some(mut c) => {
                c.name = payload.name;
                c.color = payload.color;
                c.is_primary = payload.is_primary;
                c.updated_at = Utc::now().timestamp();
                cal_repo.upsert(&c).await?;
                c.id.0
            }
            None => {
                let id = Uuid::new_v4();
                let c = crate::core::models::Calendar {
                    id: crate::core::types::DbUuid(id),
                    account_id: account.id,
                    external_id: payload.id.clone(),
                    name: payload.name,
                    color: payload.color,
                    is_primary: payload.is_primary,
                    created_at: Utc::now().timestamp(),
                    updated_at: Utc::now().timestamp(),
                };
                cal_repo.upsert(&c).await?;
                id
            }
        };
        if payload.is_primary || default_calendar_id.is_none() {
            default_calendar_id = Some(calendar_db_id);
        }
    }

    let default_calendar_id = match default_calendar_id {
        Some(id) => id,
        None => return Ok(0), // No calendars found
    };

    // Sync events for the next 30 days
    let start_time = Utc::now().timestamp();
    let end_time = start_time + 30 * 24 * 60 * 60;

    let events = calendar_provider
        .fetch_events(token, start_time, end_time)
        .await?;

    let total_items = events.len() as u32;
    let mut synced_count = 0;

    let event_repo: Box<dyn crate::core::repository::EventRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            Box::new(crate::db::sqlite::event_repository::SqliteEventRepository::new(pool.clone()))
        }
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::event_repository::PostgresEventRepository::new(pool.clone()),
        ),
    };

    let contact_repo: Box<dyn crate::core::repository::ContactRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::contact_repository::SqliteContactRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::contact_repository::PostgresContactRepository::new(pool.clone()),
        ),
    };

    let revision_repo: Box<dyn HistoricalRevisionRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::revision_repository::SqliteRevisionRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::revision_repository::PostgresRevisionRepository::new(pool.clone()),
        ),
    };

    for payload in events {
        if let Some(ref email) = payload.organizer_email {
            let contact = crate::core::models::Contact {
                id: Uuid::new_v4().into(),
                account_id: account.id,
                name: payload.organizer_name.clone(),
                email: email.clone(),
                avatar_url: None,
                notes: None,
                last_contacted_at: payload.start_time,
                created_at: chrono::Utc::now().timestamp(),
            };
            // Merged-away contacts stay merged: sync must not resurrect them.
            if !contact_repo
                .is_merged(account.id.0, &contact.email)
                .await
                .unwrap_or(false)
            {
                let _ = contact_repo.upsert(&contact).await;
            }
        }

        if let Some(ref attendees_str) = payload.attendees {
            let parsed_attendees: Vec<String> =
                serde_json::from_str(attendees_str).unwrap_or_else(|_| {
                    attendees_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                });

            for email in parsed_attendees {
                if contact_repo
                    .is_merged(account.id.0, &email)
                    .await
                    .unwrap_or(false)
                {
                    continue;
                }
                let contact = crate::core::models::Contact {
                    id: Uuid::new_v4().into(),
                    account_id: account.id,
                    name: None,
                    email,
                    avatar_url: None,
                    notes: None,
                    last_contacted_at: payload.start_time,
                    created_at: chrono::Utc::now().timestamp(),
                };
                let _ = contact_repo.upsert(&contact).await;
            }
        }

        let existing = event_repo
            .find_by_external_id(account.id.0, &payload.external_id)
            .await?;

        let conflict = existing.as_ref().is_some_and(|e| {
            !e.has_conflict
                && is_locally_modified(e.created_at, e.updated_at)
                && event_content_differs(e, &payload)
        });
        if conflict && let Some(local) = &existing {
            snapshot_conflict(revision_repo.as_ref(), "calendar_event", local.id.0, local).await;
        }

        match existing {
            Some(mut e) => {
                e.title = payload.title;
                e.description = payload.description;
                e.location = payload.location;
                e.start_time = payload.start_time;
                e.end_time = payload.end_time;
                e.is_all_day = payload.is_all_day;
                e.recurrence_rules = payload.recurrence_rules;
                e.organizer_email = payload.organizer_email;
                e.organizer_name = payload.organizer_name;
                e.attendees = payload.attendees;
                e.status = payload.status;
                e.updated_at = Utc::now().timestamp();
                e.has_conflict = e.has_conflict || conflict;
                event_repo.upsert(&e).await?;
            }
            None => {
                let e = crate::core::models::CalendarEvent {
                    id: crate::core::types::DbUuid(Uuid::new_v4()),
                    account_id: account.id,
                    calendar_id: crate::core::types::DbUuid(default_calendar_id), // Simplified: attach to default calendar
                    external_id: payload.external_id,
                    title: payload.title,
                    description: payload.description,
                    location: payload.location,
                    start_time: payload.start_time,
                    end_time: payload.end_time,
                    is_all_day: payload.is_all_day,
                    recurrence_rules: payload.recurrence_rules,
                    organizer_email: payload.organizer_email,
                    organizer_name: payload.organizer_name,
                    attendees: payload.attendees,
                    status: payload.status,
                    has_conflict: false,
                    created_at: Utc::now().timestamp(),
                    updated_at: Utc::now().timestamp(),
                };
                event_repo.upsert(&e).await?;
            }
        };
        synced_count += 1;

        if synced_count % 10 == 0 || synced_count == total_items as i64 {
            let _ = sync_tx.send(SyncEvent::SyncProgress {
                account_id: account.id.0.to_string(),
                stage: "calendars".to_string(),
                items_synced: synced_count as u32,
                total_items: Some(total_items),
            });
        }
    }

    Ok(synced_count)
}

#[cfg(test)]
mod tests {
    use super::{event_content_differs, is_locally_modified, message_content_differs};
    use crate::api::router::AppState;
    use crate::api::sync::{TokenRefresher, ensure_valid_token};
    use crate::core::models::Account;
    use crate::db::pool::DbPool;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use uuid::Uuid;

    struct MockTokenRefresher {
        should_succeed: bool,
    }

    #[async_trait::async_trait]
    impl TokenRefresher for MockTokenRefresher {
        async fn refresh(&self, _account: &Account) -> Result<serde_json::Value, String> {
            if self.should_succeed {
                Ok(serde_json::json!({
                    "access_token": "new_access_token",
                    "expires_in": 3600
                }))
            } else {
                Err("Token refresh permanently failed (e.g. revoked)".to_string())
            }
        }
    }

    // Mocking the threshold check
    #[tokio::test]
    async fn test_token_expiry_condition() {
        let mut account = Account {
            id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            user_id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            provider: "gmail".to_string(),
            provider_account_id: "test@gmail.com".to_string(),
            display_name: "Test Account".to_string(),
            access_token: Some("old_token".to_string()),
            refresh_token: Some("refresh_token".to_string()),
            token_expires_at: Some(chrono::Utc::now().timestamp() + 1000), // Valid for 1000s
            sync_error: None,
            created_at: 0,
            updated_at: 0,
        };

        // If it's valid, it shouldn't need a refresh.
        let now = chrono::Utc::now().timestamp();
        let threshold = now + 300;
        let needs_refresh =
            account.token_expires_at.is_none() || account.token_expires_at.unwrap() < threshold;
        assert_eq!(needs_refresh, false);

        // If it's within 5 minutes (300s), it needs a refresh
        account.token_expires_at = Some(now + 200);
        let needs_refresh =
            account.token_expires_at.is_none() || account.token_expires_at.unwrap() < threshold;
        assert_eq!(needs_refresh, true);

        // If it's already expired, it needs a refresh
        account.token_expires_at = Some(now - 100);
        let needs_refresh =
            account.token_expires_at.is_none() || account.token_expires_at.unwrap() < threshold;
        assert_eq!(needs_refresh, true);
    }

    async fn create_test_app_state() -> AppState {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Setup initial schema for accounts table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                provider_account_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at INTEGER,
                sync_error TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        let (sync_tx, _) = tokio::sync::broadcast::channel(100);
        let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);
        AppState {
            db: DbPool::Sqlite(pool),
            jwt_secret: "test_secret".to_string(),
            vault_secret: "test_vault_secret".to_string(),
            vault_old_secret: None,
            plugin_manager: Arc::new(RwLock::new(crate::plugins::manager::PluginManager::new())),
            sync_tx,
            sync_job_tx,
            auth_rate_limiter: crate::api::rate_limit::RateLimiter::new(
                10,
                std::time::Duration::from_secs(60),
            ),
            general_rate_limiter: crate::api::rate_limit::RateLimiter::new(
                10,
                std::time::Duration::from_secs(60),
            ),
        }
    }

    #[tokio::test]
    async fn test_ensure_valid_token_no_refresh_needed() {
        let state = create_test_app_state().await;
        let mut account = Account {
            id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            user_id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            provider: "gmail".to_string(),
            provider_account_id: "test@gmail.com".to_string(),
            display_name: "Test Account".to_string(),
            access_token: Some("old_token".to_string()),
            refresh_token: Some("refresh_token".to_string()),
            token_expires_at: Some(chrono::Utc::now().timestamp() + 1000), // Valid for 1000s
            sync_error: None,
            created_at: 0,
            updated_at: 0,
        };

        // We insert the account into the DB so updates don't fail or so we can test it properly,
        // though in ensure_valid_token, failure to update DB just logs an error.

        let refresher = MockTokenRefresher {
            should_succeed: true,
        };

        let result = ensure_valid_token(&state, &mut account, &refresher).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "old_token");
        assert_eq!(account.access_token.unwrap(), "old_token");
    }

    #[tokio::test]
    async fn test_ensure_valid_token_refresh_successful() {
        let state = create_test_app_state().await;
        let mut account = Account {
            id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            user_id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            provider: "gmail".to_string(),
            provider_account_id: "test@gmail.com".to_string(),
            display_name: "Test Account".to_string(),
            access_token: Some("old_token".to_string()),
            refresh_token: Some("refresh_token".to_string()),
            token_expires_at: Some(chrono::Utc::now().timestamp() - 100), // Expired
            sync_error: None,
            created_at: 0,
            updated_at: 0,
        };

        // Insert account so DB update works and doesn't log error
        let repo = crate::db::sqlite::account_repository::SqliteAccountRepository::new(
            match &state.db {
                DbPool::Sqlite(pool) => pool.clone(),
                _ => unreachable!(),
            },
            state.vault_secret.clone(),
        );
        crate::core::repository::AccountRepository::create(&repo, &account)
            .await
            .unwrap();

        let refresher = MockTokenRefresher {
            should_succeed: true,
        };

        let result = ensure_valid_token(&state, &mut account, &refresher).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "new_access_token");
        assert_eq!(account.access_token.unwrap(), "new_access_token");
        assert!(account.token_expires_at.unwrap() > chrono::Utc::now().timestamp());
        assert!(account.sync_error.is_none());
    }

    #[tokio::test]
    async fn test_ensure_valid_token_refresh_failed() {
        let state = create_test_app_state().await;
        let mut account = Account {
            id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            user_id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            provider: "gmail".to_string(),
            provider_account_id: "test@gmail.com".to_string(),
            display_name: "Test Account".to_string(),
            access_token: Some("old_token".to_string()),
            refresh_token: Some("refresh_token".to_string()),
            token_expires_at: Some(chrono::Utc::now().timestamp() - 100), // Expired
            sync_error: None,
            created_at: 0,
            updated_at: 0,
        };

        // Insert account
        let repo = crate::db::sqlite::account_repository::SqliteAccountRepository::new(
            match &state.db {
                DbPool::Sqlite(pool) => pool.clone(),
                _ => unreachable!(),
            },
            state.vault_secret.clone(),
        );
        crate::core::repository::AccountRepository::create(&repo, &account)
            .await
            .unwrap();

        let refresher = MockTokenRefresher {
            should_succeed: false,
        };

        let result = ensure_valid_token(&state, &mut account, &refresher).await;

        assert!(result.is_err());
        assert!(account.sync_error.is_some());
        assert_eq!(
            account.sync_error.unwrap(),
            "Token refresh permanently failed (e.g. revoked)"
        );
        // Ensure access token is left as is or updated (it doesn't clear access token, just sets sync_error)
        assert_eq!(account.access_token.unwrap(), "old_token");
    }
    #[tokio::test]
    async fn test_daemon_deduplication_and_rate_limiting() {
        let state = create_test_app_state().await;
        let (sync_tx, mut sync_rx) = tokio::sync::broadcast::channel(100);
        let (job_tx, job_rx) = tokio::sync::mpsc::channel(100);

        let account = Account {
            id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            user_id: crate::core::types::DbUuid::new(uuid::Uuid::new_v4()),
            provider: "gmail".to_string(),
            provider_account_id: "test@gmail.com".to_string(),
            display_name: "Test Account".to_string(),
            access_token: Some("old_token".to_string()),
            refresh_token: Some("refresh_token".to_string()),
            token_expires_at: Some(chrono::Utc::now().timestamp() + 1000),
            sync_error: None,
            created_at: 0,
            updated_at: 0,
        };

        // Insert into DB
        let repo = crate::db::sqlite::account_repository::SqliteAccountRepository::new(
            match &state.db {
                DbPool::Sqlite(pool) => pool.clone(),
                _ => unreachable!(),
            },
            state.vault_secret.clone(),
        );
        crate::core::repository::AccountRepository::create(&repo, &account)
            .await
            .unwrap();

        // Start daemon
        crate::api::sync::start_sync_daemon(state.clone(), sync_tx, job_rx);

        // Trigger syncs rapidly to trigger deduplication (within 30s)
        job_tx.send(account.id.0).await.unwrap();
        job_tx.send(account.id.0).await.unwrap();
        job_tx.send(account.id.0).await.unwrap();

        // Wait for the spawned tasks to have a chance to complete
        // The token is invalid and refresher will log a warning and return.
        // We assert that the database record is untouched as it should be rate-limited and deduplicated.
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let db_account =
            crate::core::repository::AccountRepository::find_by_id(&repo, account.id.0)
                .await
                .unwrap()
                .unwrap();

        // The token hasn't changed.
        assert_eq!(db_account.access_token.unwrap(), "old_token");

        // The fact that the test finishes without locking up proves the tokio::select works properly.
        // We ensure that the events processed are zero.
        let mut msg_count = 0;
        while let Ok(_msg) = sync_rx.try_recv() {
            msg_count += 1;
        }
        assert_eq!(msg_count, 4);
    }

    fn sample_message() -> crate::core::models::Message {
        crate::core::models::Message {
            id: Uuid::new_v4().into(),
            account_id: Uuid::new_v4().into(),
            external_id: "ext-1".to_string(),
            thread_id: "thread-1".to_string(),
            subject: Some("Subject".to_string()),
            sender_name: Some("Sender".to_string()),
            sender_email: "sender@example.com".to_string(),
            recipients: r#"["me@example.com"]"#.to_string(),
            date_sent: 100,
            date_received: 200,
            snippet: Some("snippet".to_string()),
            body_text: None,
            body_html: None,
            labels: Some(r#"["inbox"]"#.to_string()),
            is_read: false,
            is_archived: false,
            is_deleted: false,
            has_attachments: false,
            snoozed_until: None,
            is_set_aside: false,
            has_conflict: false,
            created_at: 1_000,
            updated_at: 2_000,
        }
    }

    fn sample_payload() -> crate::plugins::traits::MessagePayload {
        crate::plugins::traits::MessagePayload {
            id: "ext-1".to_string(),
            external_id: "ext-1".to_string(),
            thread_id: "thread-1".to_string(),
            subject: Some("Subject".to_string()),
            sender_name: Some("Sender".to_string()),
            sender_email: "sender@example.com".to_string(),
            recipients: r#"["me@example.com"]"#.to_string(),
            date_sent: 100,
            date_received: 200,
            snippet: Some("snippet".to_string()),
            labels: Some(r#"["inbox"]"#.to_string()),
            is_read: false,
        }
    }

    #[test]
    fn test_conflict_needs_local_edit_and_remote_change() {
        let local = sample_message();
        let mut remote = sample_payload();

        assert!(!message_content_differs(&local, &remote));
        assert!(is_locally_modified(local.created_at, local.updated_at));
        assert!(!is_locally_modified(local.created_at, local.created_at));

        remote.subject = Some("Remote subject".to_string());
        assert!(message_content_differs(&local, &remote));
    }

    #[test]
    fn test_event_content_comparison_ignores_unchanged_payload() {
        let local = crate::core::models::CalendarEvent {
            id: Uuid::new_v4().into(),
            account_id: Uuid::new_v4().into(),
            calendar_id: Uuid::new_v4().into(),
            external_id: "ev-1".to_string(),
            title: "Standup".to_string(),
            description: None,
            location: Some("Room".to_string()),
            start_time: 100,
            end_time: 200,
            is_all_day: false,
            recurrence_rules: None,
            organizer_email: None,
            organizer_name: None,
            attendees: None,
            status: Some("confirmed".to_string()),
            has_conflict: false,
            created_at: 1_000,
            updated_at: 2_000,
        };
        let mut remote = crate::plugins::traits::EventPayload {
            id: "ev-1".to_string(),
            external_id: "ev-1".to_string(),
            title: "Standup".to_string(),
            description: None,
            location: Some("Room".to_string()),
            start_time: 100,
            end_time: 200,
            is_all_day: false,
            recurrence_rules: None,
            organizer_email: None,
            organizer_name: None,
            attendees: None,
            status: Some("confirmed".to_string()),
        };

        assert!(!event_content_differs(&local, &remote));
        remote.start_time = 300;
        assert!(event_content_differs(&local, &remote));
    }

    #[test]
    fn test_lww_applies_newer_remote_message() {
        assert!(super::should_apply_remote_message(100, 200));
    }

    #[test]
    fn test_lww_ignores_stale_remote_message() {
        assert!(!super::should_apply_remote_message(200, 100));
    }

    #[test]
    fn test_lww_applies_equal_timestamp_remote_message() {
        assert!(super::should_apply_remote_message(100, 100));
    }
}
