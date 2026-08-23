use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use std::collections::HashMap;

use crate::api::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;

// --- Generic Router ---

pub async fn handle_generic_webhook(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    body: Option<axum::body::Bytes>,
) -> Result<impl IntoResponse, KestrelError> {
    let expected_secret = match std::env::var("WEBHOOK_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            tracing::error!("WEBHOOK_SECRET environment variable is not set. Webhooks are disabled.");
            return Err(KestrelError::Internal(Box::new(std::io::Error::other("Webhook ingestion not configured"))));
        }
    };

    let plugin_manager = state.plugin_manager.read().await;
    let plugin = match plugin_manager
        .find_by_id(&provider)
        .or_else(|| plugin_manager.find_by_provider(&provider))
    {
        Some(p) => p,
        None => {
            tracing::warn!("Webhook received for unknown provider plugin: {}", provider);
            return Err(KestrelError::NotFound("Provider not found".to_string()));
        }
    };

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
                    state.jwt_secret.clone(),
                ),
            );
            r
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let r: Box<dyn AccountRepository> = Box::new(
                crate::db::postgres::account_repository::PostgresAccountRepository::new(
                    pool.clone(),
                    state.jwt_secret.clone(),
                ),
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
