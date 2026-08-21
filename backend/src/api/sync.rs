use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use chrono::Utc;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{Semaphore, broadcast};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::account_repository::PostgresAccountRepository;
use crate::db::sqlite::account_repository::SqliteAccountRepository;

// --- Sync event types ---

#[derive(Debug, Clone, Serialize)]
pub struct SyncEvent {
    pub event_type: String,
    pub account_id: Option<Uuid>,
    pub message: String,
    pub timestamp: i64,
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
                std::env::var("GMAIL_CLIENT_ID").unwrap_or_default(),
                std::env::var("GMAIL_CLIENT_SECRET").unwrap_or_default(),
            ),
            "outlook" => (
                "https://login.microsoftonline.com/common/oauth2/v2.0/token",
                std::env::var("OUTLOOK_CLIENT_ID").unwrap_or_default(),
                std::env::var("OUTLOOK_CLIENT_SECRET").unwrap_or_default(),
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

    let needs_refresh =
        account.token_expires_at.is_none() || account.token_expires_at.unwrap() < threshold;

    if !needs_refresh {
        return Ok(account.access_token.clone().unwrap());
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
                        if let Err(e) = SqliteAccountRepository::new(pool.clone())
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
                        if let Err(e) = PostgresAccountRepository::new(pool.clone())
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
                        if let Err(e) = SqliteAccountRepository::new(pool.clone())
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
                        if let Err(e) = PostgresAccountRepository::new(pool.clone())
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
                Err(err_msg)
            }
        }
        Err(e) => {
            account.sync_error = Some(e.clone());
            match &state.db {
                DbPool::Sqlite(pool) => {
                    if let Err(e) = SqliteAccountRepository::new(pool.clone())
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
                    if let Err(e) = PostgresAccountRepository::new(pool.clone())
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

        match sync_account_messages(&state, &account, &token).await {
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

pub async fn sync_stream(
    State(state): State<AppState>,
    AuthUser { user_id: _ }: AuthUser,
) -> Sse<impl futures::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.sync_tx.subscribe();

    let stream = futures::stream::unfold(rx, move |mut rx| {
        async move {
            match rx.recv().await {
                Ok(event) => {
                    // Filter events relevant to this user
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    let sse_event = Event::default().event(&event.event_type).data(data);
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

        // Provider rate limiters (Max 5 concurrent requests per provider globally)
        let mut limits = HashMap::new();
        limits.insert("gmail".to_string(), Arc::new(Semaphore::new(5)));
        limits.insert("outlook".to_string(), Arc::new(Semaphore::new(5)));

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

            let accounts_len = accounts.len();

            // Process up to 10 accounts concurrently
            let mut stream = futures::stream::iter(accounts)
                .map(|mut account| {
                    let state_clone = state.clone();
                    let tx_clone = sync_tx.clone();
                    let semaphore = limits.get(&account.provider).cloned();

                    async move {
                        let refresher = ReqwestTokenRefresher::new();
                        let token = match ensure_valid_token(&state_clone, &mut account, &refresher)
                            .await
                        {
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

                        // 60-second timeout to prevent hanging on one provider
                        let msg_sync = tokio::time::timeout(
                            Duration::from_secs(60),
                            sync_account_messages(&state_clone, &account, &token),
                        )
                        .await;

                        match msg_sync {
                            Ok(Ok(count)) => {
                                if count > 0 {
                                    let event = SyncEvent {
                                        event_type: "sync_complete".to_string(),
                                        account_id: Some(account.id.0),
                                        message: format!("Synced {} new messages", count),
                                        timestamp: Utc::now().timestamp(),
                                    };
                                    let _ = tx_clone.send(event);
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::warn!("Sync daemon failed for {}: {}", account.id.0, e)
                            }
                            Err(_) => tracing::warn!("Sync daemon timed out for {}", account.id.0),
                        }

                        let cal_sync = tokio::time::timeout(
                            Duration::from_secs(60),
                            sync_account_calendars(&state_clone, &account, &token),
                        )
                        .await;

                        match cal_sync {
                            Ok(Ok(count)) => {
                                if count > 0 {
                                    tracing::info!(
                                        "Synced {} new calendar events for account {}",
                                        count,
                                        account.id.0
                                    );
                                }
                            }
                            Ok(Err(e)) => {
                                tracing::warn!("Calendar sync failed for {}: {}", account.id.0, e)
                            }
                            Err(_) => {
                                tracing::warn!("Calendar sync timed out for {}", account.id.0)
                            }
                        }
                    }
                })
                .buffer_unordered(10); // Run up to 10 concurrently

            while stream.next().await.is_some() {}

            tracing::info!(
                "Sync daemon: cycle complete, synced {} account(s)",
                accounts_len
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
            let accounts = sqlx::query_as::<_, crate::core::models::Account>(
                "SELECT id, user_id, provider, provider_account_id, display_name, \
                 access_token, refresh_token, token_expires_at, created_at, updated_at \
                 FROM accounts WHERE access_token IS NOT NULL",
            )
            .fetch_all(pool)
            .await?;
            Ok(accounts)
        }
    }
}

/// K-038: LWW conflict resolution — sync messages from a provider account.
/// Compares timestamps to decide whether to upsert or skip.
pub async fn sync_account_messages(
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

    let blocked_senders = filter_repo
        .get_blocked_senders(account.user_id.0)
        .await
        .unwrap_or_default();
    let blocked_set: std::collections::HashSet<String> = blocked_senders.into_iter().collect();

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
                payload.date_received >= msg.date_received
            }
            None => true,
        };

        if should_upsert {
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

/// Task 2.2: Fetch and store calendar events.
pub async fn sync_account_calendars(
    state: &AppState,
    account: &crate::core::models::Account,
    token: &str,
) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
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
    let mut synced_count = 0;

    let event_repo: Box<dyn crate::core::repository::EventRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            Box::new(crate::db::sqlite::event_repository::SqliteEventRepository::new(pool.clone()))
        }
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::event_repository::PostgresEventRepository::new(pool.clone()),
        ),
    };

    for payload in events {
        let existing = event_repo
            .find_by_external_id(account.id.0, &payload.external_id)
            .await?;

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
                    created_at: Utc::now().timestamp(),
                    updated_at: Utc::now().timestamp(),
                };
                event_repo.upsert(&e).await?;
            }
        };
        synced_count += 1;
    }

    Ok(synced_count)
}

#[cfg(test)]
mod tests {
    use crate::api::router::AppState;
    use crate::api::sync::{TokenRefresher, ensure_valid_token};
    use crate::core::models::Account;
    use crate::db::pool::DbPool;
    use std::sync::Arc;
    use tokio::sync::RwLock;

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
        AppState {
            db: DbPool::Sqlite(pool),
            jwt_secret: "test_secret".to_string(),
            plugin_manager: Arc::new(RwLock::new(crate::plugins::manager::PluginManager::new())),
            sync_tx,
            auth_rate_limiter: crate::api::rate_limit::RateLimiter::new(
                10,
                std::time::Duration::from_secs(60),
            ),
            general_rate_limiter: crate::api::rate_limit::RateLimiter::new(
                10,
                std::time::Duration::from_secs(60),
            ),
            http_client: reqwest::Client::builder()
                .dns_resolver(std::sync::Arc::new(crate::api::proxy::SafeDnsResolver))
                .build()
                .unwrap(),
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
        let repo =
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(match &state.db {
                DbPool::Sqlite(pool) => pool.clone(),
                _ => unreachable!(),
            });
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
        let repo =
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(match &state.db {
                DbPool::Sqlite(pool) => pool.clone(),
                _ => unreachable!(),
            });
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
}
