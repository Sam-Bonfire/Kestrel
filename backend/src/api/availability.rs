use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;

#[derive(Debug, Deserialize, specta::Type)]
pub struct FreebusyRequest {
    pub emails: Vec<String>,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct BusyBlockDto {
    pub email: String,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
}

fn plugin_error(e: crate::plugins::traits::PluginError) -> KestrelError {
    let message = format!("Plugin error: {:?}", e);
    let code = message
        .split("HTTP ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|digits| digits.parse::<u16>().ok());
    match code {
        Some(401) => KestrelError::Unauthorized,
        Some(404) => KestrelError::NotFound(message),
        _ => {
            tracing::warn!("{}", message);
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(message)))
        }
    }
}

/// Teammate free/busy blocks from the account's calendar provider.
pub async fn query_freebusy(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(body): Json<FreebusyRequest>,
) -> Result<Json<Vec<BusyBlockDto>>, KestrelError> {
    if body.emails.is_empty() || body.emails.len() > 20 {
        return Err(KestrelError::BadRequest("Provide 1-20 emails".to_string()));
    }
    if body.start_time >= body.end_time {
        return Err(KestrelError::BadRequest(
            "start_time must be before end_time".to_string(),
        ));
    }
    if body.end_time - body.start_time > 31 * 24 * 3600 {
        return Err(KestrelError::BadRequest(
            "Range exceeds 31 days".to_string(),
        ));
    }

    let account_repo: Box<dyn AccountRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            ),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            ),
        ),
    };
    let account = account_repo
        .find_by_user_id(user_id)
        .await?
        .into_iter()
        .find(|a| a.id.0 == account_id)
        .ok_or_else(|| KestrelError::NotFound("Account not found".to_string()))?;
    let token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    let manager = state.plugin_manager.read().await;
    let plugin = manager.find_by_provider(&account.provider).ok_or_else(|| {
        KestrelError::BadRequest(format!("No plugin for provider {}", account.provider))
    })?;
    let blocks = plugin
        .as_calendar_provider()
        .query_freebusy(&token, &body.emails, body.start_time, body.end_time)
        .await
        .map_err(plugin_error)?;

    Ok(Json(
        blocks
            .into_iter()
            .map(|b| BusyBlockDto {
                email: b.email,
                start_time: b.start_time,
                end_time: b.end_time,
            })
            .collect(),
    ))
}
