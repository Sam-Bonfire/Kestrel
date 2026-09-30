use axum::{Json, extract::State};

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::SettingsPayload;
use crate::core::repository::UserPreferencesRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::user_preferences_repository::PostgresUserPreferencesRepository;
use crate::db::sqlite::user_preferences_repository::SqliteUserPreferencesRepository;

pub async fn get_settings(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<SettingsPayload>, KestrelError> {
    let settings_str = match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteUserPreferencesRepository::new(pool.clone())
                .get_preferences(user_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            PostgresUserPreferencesRepository::new(pool.clone())
                .get_preferences(user_id)
                .await?
        }
    };

    let prefs: SettingsPayload = if let Some(prefs) = settings_str {
        serde_json::from_str(&prefs.preferences_json).unwrap_or_default()
    } else {
        SettingsPayload::default()
    };

    Ok(Json(prefs))
}

pub async fn update_settings(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<SettingsPayload>,
) -> Result<Json<SettingsPayload>, KestrelError> {
    // First, fetch current settings to merge
    let current_str = match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteUserPreferencesRepository::new(pool.clone())
                .get_preferences(user_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            PostgresUserPreferencesRepository::new(pool.clone())
                .get_preferences(user_id)
                .await?
        }
    };

    let mut current_prefs: SettingsPayload = if let Some(prefs) = current_str {
        serde_json::from_str(&prefs.preferences_json).unwrap_or_default()
    } else {
        SettingsPayload::default()
    };

    if payload.mail_dense_mode.is_some() {
        current_prefs.mail_dense_mode = payload.mail_dense_mode;
    }
    if payload.mail_default_landing_view.is_some() {
        current_prefs.mail_default_landing_view = payload.mail_default_landing_view;
    }
    if payload.mail_signature.is_some() {
        current_prefs.mail_signature = payload.mail_signature;
    }
    if payload.mail_snooze_default.is_some() {
        current_prefs.mail_snooze_default = payload.mail_snooze_default;
    }
    if payload.swipe_left_action.is_some() {
        current_prefs.swipe_left_action = payload.swipe_left_action;
    }
    if payload.swipe_right_action.is_some() {
        current_prefs.swipe_right_action = payload.swipe_right_action;
    }
    if payload.label_customizations.is_some() {
        current_prefs.label_customizations = payload.label_customizations;
    }
    if payload.sync_interval.is_some() {
        current_prefs.sync_interval = payload.sync_interval;
    }
    if payload.theme.is_some() {
        current_prefs.theme = payload.theme;
    }
    if payload.snippets.is_some() {
        current_prefs.snippets = payload.snippets;
    }
    if payload.signatures.is_some() {
        current_prefs.signatures = payload.signatures;
    }
    if let Some(mut prefs) = payload.notification_prefs {
        if let Some(s) = &prefs.quiet_hours_start {
            if !is_valid_hhmm(s) {
                return Err(KestrelError::BadRequest(format!(
                    "quiet_hours_start must be HH:MM, got '{s}'"
                )));
            }
        }
        if let Some(s) = &prefs.quiet_hours_end {
            if !is_valid_hhmm(s) {
                return Err(KestrelError::BadRequest(format!(
                    "quiet_hours_end must be HH:MM, got '{s}'"
                )));
            }
        }
        prefs.loud_contacts = normalize_list(&prefs.loud_contacts);
        prefs.loud_threads = normalize_list(&prefs.loud_threads);
        current_prefs.notification_prefs = Some(prefs);
    }
    if payload.ai_toggles.is_some() {
        current_prefs.ai_toggles = payload.ai_toggles;
    }

    let new_prefs_str = serde_json::to_string(&current_prefs).map_err(|e| {
        KestrelError::Internal(format!("Failed to serialize preferences: {}", e).into())
    })?;

    match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteUserPreferencesRepository::new(pool.clone())
                .update_preferences(user_id, &new_prefs_str)
                .await?;
        }
        DbPool::Postgres(pool) => {
            PostgresUserPreferencesRepository::new(pool.clone())
                .update_preferences(user_id, &new_prefs_str)
                .await?;
        }
    }

    Ok(Json(current_prefs))
}

// ponytail: naive HH:MM check, chrono parse if formats ever widen
fn is_valid_hhmm(s: &str) -> bool {
    match s.split_once(':') {
        Some((h, m)) => {
            h.len() == 2
                && m.len() == 2
                && matches!(h.parse::<u8>(), Ok(h) if h < 24)
                && matches!(m.parse::<u8>(), Ok(m) if m < 60)
        }
        None => false,
    }
}

fn normalize_list(items: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    items
        .iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .collect()
}
