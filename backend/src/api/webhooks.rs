use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use std::collections::HashMap;

use crate::api::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;

// --- Generic Router ---

const SHARED_WEBHOOK_SECRET_ENV: &str = "WEBHOOK_SECRET";

/// Environment key for a provider-specific secret, e.g. `WEBHOOK_SECRET_GOOGLE_MAIL`
/// for provider `google-mail`.
pub(crate) fn provider_secret_env_key(provider: &str) -> String {
    let mut key = String::from("WEBHOOK_SECRET_");
    for c in provider.chars() {
        key.push(if c.is_ascii_alphanumeric() {
            c.to_ascii_uppercase()
        } else {
            '_'
        });
    }
    key
}

/// Resolve the webhook secret for a provider.
///
/// Provider-specific keys win. The shared `WEBHOOK_SECRET` remains as a
/// backwards-compatible fallback for existing self-hosters.
pub(crate) fn resolve_webhook_secret(
    provider: &str,
    lookup: impl Fn(&str) -> Option<String>,
) -> Option<String> {
    [
        provider_secret_env_key(provider),
        SHARED_WEBHOOK_SECRET_ENV.to_string(),
    ]
    .into_iter()
    .filter_map(|key| lookup(&key))
    .find(|value| !value.trim().is_empty())
}

pub async fn handle_generic_webhook(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    body: Option<axum::body::Bytes>,
) -> Result<impl IntoResponse, KestrelError> {
    let plugin_manager = state.plugin_manager.read().await;
    let plugin = match plugin_manager
        .find_by_id(&provider)
        .or_else(|| plugin_manager.find_by_provider(&provider))
    {
        Some(p) => p,
        None => {
            tracing::warn!(
                "Webhook received for unknown or unloaded provider plugin: {}",
                provider
            );
            return Err(KestrelError::NotFound(
                "Webhook endpoint not found".to_string(),
            ));
        }
    };

    let expected_secret = resolve_webhook_secret(&provider, |key| std::env::var(key).ok())
        .ok_or_else(|| {
            tracing::error!(
                "Webhook secret not configured for provider '{}'; rejecting delivery",
                provider
            );
            KestrelError::Unavailable("Webhook secret not configured".to_string())
        })?;

    let bytes = body.map(|b| b.to_vec()).unwrap_or_default();

    let query_vec: Vec<(String, String)> = query_params.into_iter().collect();

    let handler = plugin.as_webhook_handler();
    let result = match handler
        .handle_webhook(&expected_secret, query_vec, bytes)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Webhook handling failed for plugin {}: {}", provider, e);
            return Err(KestrelError::BadRequest(
                "Webhook rejected by plugin".to_string(),
            ));
        }
    };

    // Construct the response
    use axum::response::Response;
    let mut builder = Response::builder().status(result.status);
    for (k, v) in result.headers {
        builder = builder.header(k, v);
    }
    let response = builder.body(axum::body::Body::from(result.body)).unwrap();

    // Extract email from the parsed payload
    let email = match result.account_identifier {
        Some(ref e) if !e.is_empty() => e.clone(),
        _ => return Ok(response.into_response()),
    };

    tracing::info!("Received webhook for email: {}", email);

    let repo = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            let r: Box<dyn AccountRepository> = Box::new(
                crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                    pool.clone(),
                    state.vault_secret.clone(),
                )
                .with_fallback_key(state.vault_old_secret.clone()),
            );
            r
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let r: Box<dyn AccountRepository> = Box::new(
                crate::db::postgres::account_repository::PostgresAccountRepository::new(
                    pool.clone(),
                    state.vault_secret.clone(),
                )
                .with_fallback_key(state.vault_old_secret.clone()),
            );
            r
        }
    };

    let account_opt = repo
        .find_by_provider_account_id(&provider, &email)
        .await
        .map_err(|e| {
            tracing::error!("DB error finding account for webhook: {}", e);
            e
        })?;

    let account = match account_opt {
        Some(a) => a,
        None => {
            tracing::warn!("No account found for webhook email: {}", email);
            return Ok(response.into_response());
        }
    };

    // Enqueue sync job via the sync daemon channel
    let account_id = account.id.0;

    if let Err(e) = state.sync_job_tx.send(account_id).await {
        tracing::error!(
            "Failed to enqueue sync job for account {}: {}",
            account_id,
            e
        );
    } else {
        tracing::info!(
            "Enqueued background sync job for webhook account: {}",
            account_id
        );
    }

    Ok(response.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup<'a>(map: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            map.iter()
                .find(|(candidate, _)| *candidate == key)
                .map(|(_, value)| value.to_string())
        }
    }

    #[test]
    fn provider_specific_secret_wins_over_shared() {
        let map = [
            ("WEBHOOK_SECRET", "shared"),
            ("WEBHOOK_SECRET_GOOGLE_MAIL", "per-provider"),
        ];
        assert_eq!(
            resolve_webhook_secret("google-mail", lookup(&map)).as_deref(),
            Some("per-provider")
        );
    }

    #[test]
    fn falls_back_to_shared_secret() {
        let map = [("WEBHOOK_SECRET", "shared")];
        assert_eq!(
            resolve_webhook_secret("gmail", lookup(&map)).as_deref(),
            Some("shared")
        );
    }

    #[test]
    fn ignores_blank_secrets() {
        let map = [("WEBHOOK_SECRET", "  "), ("WEBHOOK_SECRET_GMAIL", "")];
        assert_eq!(resolve_webhook_secret("gmail", lookup(&map)), None);
    }

    #[test]
    fn missing_secret_fails_closed() {
        assert_eq!(resolve_webhook_secret("gmail", lookup(&[])), None);
    }

    #[test]
    fn provider_key_is_normalized() {
        assert_eq!(
            provider_secret_env_key("google-mail"),
            "WEBHOOK_SECRET_GOOGLE_MAIL"
        );
        assert_eq!(provider_secret_env_key("Outlook"), "WEBHOOK_SECRET_OUTLOOK");
    }
}
