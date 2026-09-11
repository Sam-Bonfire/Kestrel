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

#[derive(Debug, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VacationDto {
    pub enabled: bool,
    pub subject: Option<String>,
    pub body_text: String,
    #[specta(type = Option<f64>)]
    pub start_time: Option<i64>,
    #[specta(type = Option<f64>)]
    pub end_time: Option<i64>,
}

#[derive(Debug, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct VacationUpdate {
    pub enabled: bool,
    pub subject: Option<String>,
    pub body_text: String,
    #[specta(type = Option<f64>)]
    pub start_time: Option<i64>,
    #[specta(type = Option<f64>)]
    pub end_time: Option<i64>,
}

async fn account_token(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<(String, String), KestrelError> {
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
    Ok((account.provider, token))
}

fn plugin_error(e: crate::plugins::traits::PluginError) -> KestrelError {
    let message = format!("Plugin error: {:?}", e);
    // Provider failures surface as "HTTP <code>"; preserve the status.
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

pub async fn get_vacation(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<Json<VacationDto>, KestrelError> {
    let (provider_name, token) = account_token(&state, user_id, account_id).await?;
    let manager = state.plugin_manager.read().await;
    let plugin = manager.find_by_provider(&provider_name).ok_or_else(|| {
        KestrelError::BadRequest(format!("No plugin for provider {}", provider_name))
    })?;
    let settings = plugin
        .as_mail_provider()
        .get_vacation(&token)
        .await
        .map_err(plugin_error)?;
    Ok(Json(VacationDto {
        enabled: settings.enabled,
        subject: settings.subject,
        body_text: settings.body_text,
        start_time: settings.start_time,
        end_time: settings.end_time,
    }))
}

fn validate_update(body: &VacationUpdate) -> Result<(), KestrelError> {
    if body.body_text.chars().count() > 10000 {
        return Err(KestrelError::BadRequest(
            "Auto-reply body exceeds 10000 characters".to_string(),
        ));
    }
    match (body.start_time, body.end_time) {
        (Some(start), Some(end)) if start >= end => {
            return Err(KestrelError::BadRequest(
                "Vacation start must be before end".to_string(),
            ));
        }
        (Some(_), None) | (None, Some(_)) => {
            return Err(KestrelError::BadRequest(
                "Vacation needs both start and end, or neither".to_string(),
            ));
        }
        _ => {}
    }
    Ok(())
}

pub async fn set_vacation(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(account_id): Path<Uuid>,
    Json(body): Json<VacationUpdate>,
) -> Result<Json<serde_json::Value>, KestrelError> {
    validate_update(&body)?;
    let (provider_name, token) = account_token(&state, user_id, account_id).await?;
    let manager = state.plugin_manager.read().await;
    let plugin = manager.find_by_provider(&provider_name).ok_or_else(|| {
        KestrelError::BadRequest(format!("No plugin for provider {}", provider_name))
    })?;
    plugin
        .as_mail_provider()
        .set_vacation(
            &token,
            crate::plugins::traits::VacationSettings {
                enabled: body.enabled,
                subject: body.subject,
                body_text: body.body_text,
                start_time: body.start_time,
                end_time: body.end_time,
            },
        )
        .await
        .map_err(plugin_error)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(enabled: bool, start: Option<i64>, end: Option<i64>) -> VacationUpdate {
        VacationUpdate {
            enabled,
            subject: None,
            body_text: "Away".to_string(),
            start_time: start,
            end_time: end,
        }
    }

    #[test]
    fn test_validate_update() {
        assert!(validate_update(&update(true, Some(1), Some(2))).is_ok());
        assert!(validate_update(&update(true, None, None)).is_ok());
        assert!(validate_update(&update(true, Some(2), Some(1))).is_err());
        assert!(validate_update(&update(true, Some(1), None)).is_err());
        assert!(validate_update(&update(true, None, Some(2))).is_err());
        assert!(
            validate_update(&VacationUpdate {
                body_text: "x".repeat(10001),
                ..update(true, None, None)
            })
            .is_err()
        );
    }
}
