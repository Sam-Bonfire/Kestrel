use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::ThreadSubjectOverride;
use crate::core::repository::{AccountRepository, ThreadSubjectOverrideRepository};
use crate::db::pool::DbPool;

fn override_repo(state: &AppState) -> Box<dyn ThreadSubjectOverrideRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::override_repository::SqliteThreadSubjectOverrideRepository::new(
                pool.clone(),
            ),
        ),
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::override_repository::PostgresThreadSubjectOverrideRepository::new(
                pool.clone(),
            ),
        ),
    }
}

async fn verify_account_owner(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<(), KestrelError> {
    let account = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )
            .find_by_id(account_id)
            .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )
            .find_by_id(account_id)
            .await?
        }
    };
    match account {
        Some(a) if a.user_id.0 == user_id => Ok(()),
        _ => Err(KestrelError::NotFound("Account not found".to_string())),
    }
}

fn validate_thread_id(thread_id: &str) -> Result<(), KestrelError> {
    if thread_id.is_empty() || thread_id.len() > 256 {
        return Err(KestrelError::BadRequest("Invalid thread id".to_string()));
    }
    Ok(())
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct SetSubjectRequest {
    pub account_id: Uuid,
    pub subject: String,
}

/// PUT /api/v1/threads/:thread_id/subject — personal display rename.
/// Local-only; the provider subject is never touched.
pub async fn set_subject(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(thread_id): Path<String>,
    Json(body): Json<SetSubjectRequest>,
) -> Result<StatusCode, KestrelError> {
    let subject = body.subject.trim().to_string();
    if subject.is_empty() || subject.chars().count() > 200 {
        return Err(KestrelError::BadRequest(
            "subject must be 1-200 characters".to_string(),
        ));
    }
    validate_thread_id(&thread_id)?;
    verify_account_owner(&state, user_id, body.account_id).await?;
    override_repo(&state)
        .upsert(body.account_id, &thread_id, &subject)
        .await
        .map_err(KestrelError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/threads/:thread_id/subject?account_id=... — restore provider subject.
pub async fn clear_subject(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(thread_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, KestrelError> {
    let account_id: Uuid = params
        .get("account_id")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| KestrelError::BadRequest("account_id is required".to_string()))?;
    validate_thread_id(&thread_id)?;
    verify_account_owner(&state, user_id, account_id).await?;
    override_repo(&state)
        .delete(account_id, &thread_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/threads/overrides?account_id=... — map for display overlay.
pub async fn list_overrides(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ThreadSubjectOverride>>, KestrelError> {
    let account_id: Uuid = params
        .get("account_id")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| KestrelError::BadRequest("account_id is required".to_string()))?;
    verify_account_owner(&state, user_id, account_id).await?;
    let rows = override_repo(&state)
        .list_by_account(account_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(rows))
}
