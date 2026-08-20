use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::Engine;
use chrono::Utc;
use serde::Deserialize;

use crate::api::router::AppState;
use crate::api::sync::{SyncEvent, sync_account_messages};
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;

#[derive(Debug, Deserialize)]
pub struct GoogleWebhookQuery {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GooglePubSubMessage {
    pub data: String, // base64 encoded JSON
}

#[derive(Debug, Deserialize)]
pub struct GoogleWebhookPayload {
    pub message: GooglePubSubMessage,
}

#[derive(Debug, Deserialize)]
pub struct GoogleWebhookData {
    #[serde(rename = "emailAddress")]
    pub email_address: String,
}

pub async fn handle_google_webhook(
    State(state): State<AppState>,
    Query(query): Query<GoogleWebhookQuery>,
    Json(payload): Json<GoogleWebhookPayload>,
) -> Result<impl IntoResponse, KestrelError> {
    let expected_secret =
        std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| state.jwt_secret.clone());

    // Verify the authentication token
    if query.token.as_deref() != Some(expected_secret.as_str()) {
        tracing::warn!("Google webhook failed signature verification");
        return Err(KestrelError::Unauthorized);
    }

    let decoded_data = base64::engine::general_purpose::STANDARD
        .decode(payload.message.data)
        .map_err(|e| {
            tracing::warn!("Failed to decode Google webhook base64 data: {}", e);
            KestrelError::BadRequest("Invalid base64 payload".to_string())
        })?;

    let data: GoogleWebhookData = serde_json::from_slice(&decoded_data).map_err(|e| {
        tracing::warn!("Failed to parse Google webhook data JSON: {}", e);
        KestrelError::BadRequest("Invalid JSON payload".to_string())
    })?;

    let email = data.email_address;
    tracing::info!("Received Google webhook for email: {}", email);

    let repo = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            let r: Box<dyn AccountRepository> = Box::new(
                crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone()),
            );
            r
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let r: Box<dyn AccountRepository> = Box::new(
                crate::db::postgres::account_repository::PostgresAccountRepository::new(
                    pool.clone(),
                ),
            );
            r
        }
    };

    let account_opt = repo
        .find_by_provider_account_id("gmail", &email)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding account for webhook: {}", e);
            e
        })?;

    let mut account = match account_opt {
        Some(a) => a,
        None => {
            tracing::warn!("No account found for Google webhook email: {}", email);
            return Ok((StatusCode::OK, "OK").into_response());
        }
    };

    // Trigger sync job in background
    let state_clone = state.clone();
    let account_id = account.id.0;

    // Spawn task to enqueue sync job via tx
    tokio::spawn(async move {
        tracing::info!(
            "Starting background sync for Google webhook account: {}",
            account_id
        );

        let refresher = crate::api::sync::ReqwestTokenRefresher::new();
        let token = match crate::api::sync::ensure_valid_token(
            &state_clone,
            &mut account,
            &refresher,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("Webhook token refresh failed for {}: {}", account_id, e);
                return;
            }
        };

        match sync_account_messages(&state_clone, &account, &token).await {
            Ok(count) => {
                tracing::info!(
                    "Webhook sync complete for {}, {} messages",
                    account_id,
                    count
                );
                let _ = state_clone.sync_tx.send(SyncEvent {
                    event_type: "sync_complete".to_string(),
                    account_id: Some(account_id),
                    message: format!("Webhook sync complete: {} messages", count),
                    timestamp: Utc::now().timestamp(),
                });
            }
            Err(e) => {
                tracing::warn!("Webhook sync failed for {}: {}", account_id, e);
            }
        }
    });

    Ok((StatusCode::OK, "OK").into_response())
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftWebhookQuery {
    #[serde(rename = "validationToken")]
    pub validation_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftNotification {
    #[serde(rename = "clientState")]
    pub client_state: Option<String>,
    pub resource: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftWebhookPayload {
    pub value: Vec<MicrosoftNotification>,
}

pub async fn handle_microsoft_webhook(
    State(state): State<AppState>,
    Query(query): Query<MicrosoftWebhookQuery>,
    body: Option<axum::body::Bytes>,
) -> Result<impl IntoResponse, KestrelError> {
    // 1. Handle validation handshake
    if let Some(token) = query.validation_token {
        tracing::info!("Microsoft webhook validation handshake");
        // Must return text/plain for validation token
        use axum::response::Response;
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(axum::body::Body::from(token))
            .unwrap();
        return Ok(response.into_response());
    }

    let expected_secret =
        std::env::var("WEBHOOK_SECRET").unwrap_or_else(|_| state.jwt_secret.clone());

    // 2. Handle actual notification
    if let Some(bytes) = body {
        if bytes.is_empty() {
            return Ok((StatusCode::ACCEPTED, "Accepted").into_response());
        }

        let payload: MicrosoftWebhookPayload = match serde_json::from_slice(&bytes) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to parse Microsoft webhook payload: {}", e);
                return Err(KestrelError::BadRequest("Invalid JSON".to_string()));
            }
        };

        for notification in payload.value {
            // Verify the client state matches our expected webhook secret
            if notification.client_state.as_deref() != Some(expected_secret.as_str()) {
                tracing::warn!("Microsoft webhook failed signature/clientState verification");
                return Err(KestrelError::Unauthorized);
            }

            // We expect the provider_account_id (e.g. email) to be in clientState
            // wait, if client_state is the secret, then it can't contain the email.
            // Let's parse the email from the resource string instead.
            let email = extract_email_from_resource(notification.resource.as_deref());

            if email.is_empty() {
                tracing::warn!("Could not extract email from Microsoft notification resource");
                continue;
            }

            tracing::info!("Received Microsoft webhook for email: {}", email);

            let repo = match &state.db {
                crate::db::pool::DbPool::Sqlite(pool) => {
                    let r: Box<dyn AccountRepository> = Box::new(
                        crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                            pool.clone(),
                        ),
                    );
                    r
                }
                crate::db::pool::DbPool::Postgres(pool) => {
                    let r: Box<dyn AccountRepository> = Box::new(
                        crate::db::postgres::account_repository::PostgresAccountRepository::new(
                            pool.clone(),
                        ),
                    );
                    r
                }
            };

            let account_opt = match repo.find_by_provider_account_id("outlook", &email).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("DB error finding account for webhook: {}", e);
                    continue;
                }
            };

            let mut account = match account_opt {
                Some(a) => a,
                None => {
                    // It's possible the provider is "microsoft", try that
                    match repo.find_by_provider_account_id("microsoft", &email).await {
                        Ok(Some(a)) => a,
                        _ => {
                            tracing::warn!(
                                "No account found for Microsoft webhook email: {}",
                                email
                            );
                            continue;
                        }
                    }
                }
            };

            let state_clone = state.clone();
            let account_id = account.id.0;

            tokio::spawn(async move {
                tracing::info!(
                    "Starting background sync for Microsoft webhook account: {}",
                    account_id
                );
                let refresher = crate::api::sync::ReqwestTokenRefresher::new();
                let token = match crate::api::sync::ensure_valid_token(
                    &state_clone,
                    &mut account,
                    &refresher,
                )
                .await
                {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::warn!("Webhook token refresh failed for {}: {}", account_id, e);
                        return;
                    }
                };

                match sync_account_messages(&state_clone, &account, &token).await {
                    Ok(count) => {
                        tracing::info!(
                            "Webhook sync complete for {}, {} messages",
                            account_id,
                            count
                        );
                        let _ = state_clone.sync_tx.send(SyncEvent {
                            event_type: "sync_complete".to_string(),
                            account_id: Some(account_id),
                            message: format!("Webhook sync complete: {} messages", count),
                            timestamp: Utc::now().timestamp(),
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Webhook sync failed for {}: {}", account_id, e);
                    }
                }
            });
        }
    }

    // Always respond 202 Accepted quickly for Microsoft notifications
    Ok((StatusCode::ACCEPTED, "Accepted").into_response())
}

fn extract_email_from_resource(resource: Option<&str>) -> String {
    let r = resource.unwrap_or("");
    // Expecting something like "Users/user@domain.com/Messages" or "users('user@domain.com')/..."
    if r.starts_with("Users/") || r.starts_with("users/") {
        let parts: Vec<&str> = r.split('/').collect();
        if parts.len() > 1 {
            let email_part = parts[1];
            // Remove quotes if present
            let clean = email_part.trim_matches(|c| c == '\'' || c == '"' || c == '(' || c == ')');
            if clean.contains('@') {
                return clean.to_string();
            }
        }
    } else if let Some(start) = r.find("('") {
        let slice = &r[start + 2..];
        if let Some(end) = slice.find("')") {
            let possible_email = &slice[..end];
            if possible_email.contains('@') {
                return possible_email.to_string();
            }
        }
    }

    // In case it's literally just the email
    if r.contains('@') && !r.contains('/') {
        return r.to_string();
    }

    "".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::pool::DbPool;
    use axum::http::StatusCode;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_app_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE accounts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                provider_account_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at BIGINT,
                sync_error TEXT,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            );",
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
        }
    }

    #[tokio::test]
    async fn test_microsoft_webhook_handshake() {
        let state = create_test_app_state().await;

        let query = MicrosoftWebhookQuery {
            validation_token: Some("test_token_123".to_string()),
        };

        let response = handle_microsoft_webhook(State(state), Query(query), None)
            .await
            .unwrap()
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_extract_email_from_resource() {
        assert_eq!(
            extract_email_from_resource(Some("Users/user@domain.com/Messages")),
            "user@domain.com"
        );
        assert_eq!(
            extract_email_from_resource(Some("users('user@domain.com')/Messages")),
            "user@domain.com"
        );
        assert_eq!(
            extract_email_from_resource(Some("user@domain.com")),
            "user@domain.com"
        );
    }
}

#[cfg(test)]
mod payload_tests {
    use super::*;
    use crate::db::pool::DbPool;
    use axum::http::StatusCode;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    async fn create_test_app_state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE accounts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                provider_account_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                access_token TEXT,
                refresh_token TEXT,
                token_expires_at BIGINT,
                sync_error TEXT,
                created_at BIGINT NOT NULL,
                updated_at BIGINT NOT NULL
            );",
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
        }
    }

    #[tokio::test]
    async fn test_google_webhook_invalid_token() {
        let state = create_test_app_state().await;

        let query = GoogleWebhookQuery {
            token: Some("invalid_token".to_string()),
        };

        let payload = GoogleWebhookPayload {
            message: GooglePubSubMessage {
                data: "ewogICJlbWFpbEFkZHJlc3MiOiAidGVzdEBnbWFpbC5jb20iLAogICJoaXN0b3J5SWQiOiAxMjM0NQp9".to_string(),
            }
        };

        let result = handle_google_webhook(State(state), Query(query), Json(payload)).await;
        match result {
            Err(KestrelError::Unauthorized) => {}
            _ => panic!("Expected Unauthorized"),
        }
    }

    #[tokio::test]
    async fn test_microsoft_webhook_invalid_client_state() {
        let state = create_test_app_state().await;

        let query = MicrosoftWebhookQuery {
            validation_token: None,
        };

        let payload = r#"
        {
            "value": [
                {
                    "clientState": "wrong_secret",
                    "resource": "Users/test@outlook.com/Messages"
                }
            ]
        }
        "#;

        let bytes = axum::body::Bytes::from(payload);
        let result = handle_microsoft_webhook(State(state), Query(query), Some(bytes)).await;
        match result {
            Err(KestrelError::Unauthorized) => {}
            _ => panic!("Expected Unauthorized"),
        }
    }
}
