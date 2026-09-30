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
use crate::core::models::{Clip, ThreadNote};
use crate::core::repository::{AccountRepository, ClipRepository, ThreadNoteRepository};
use crate::core::types::DbUuid;
use crate::db::pool::DbPool;

fn clip_repo(state: &AppState) -> Box<dyn ClipRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            Box::new(crate::db::sqlite::clip_repository::SqliteClipRepository::new(pool.clone()))
        }
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::clip_repository::PostgresClipRepository::new(pool.clone()),
        ),
    }
}

fn note_repo(state: &AppState) -> Box<dyn ThreadNoteRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::clip_repository::SqliteThreadNoteRepository::new(pool.clone()),
        ),
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::clip_repository::PostgresThreadNoteRepository::new(pool.clone()),
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
    if thread_id.is_empty() || thread_id.chars().count() > 256 {
        return Err(KestrelError::BadRequest("Invalid thread id".to_string()));
    }
    Ok(())
}

// --- Clips ---

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateClipRequest {
    pub account_id: Uuid,
    pub message_id: String,
    pub snippet: String,
}

/// POST /api/v1/clips — save a clipped span with its source message.
pub async fn create_clip(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<CreateClipRequest>,
) -> Result<(StatusCode, Json<Clip>), KestrelError> {
    let snippet = body.snippet.trim().to_string();
    if snippet.is_empty() || snippet.chars().count() > 2000 {
        return Err(KestrelError::BadRequest(
            "snippet must be 1-2000 characters".to_string(),
        ));
    }
    if body.message_id.is_empty() || body.message_id.chars().count() > 256 {
        return Err(KestrelError::BadRequest("Invalid message id".to_string()));
    }
    verify_account_owner(&state, user_id, body.account_id).await?;
    let now = chrono::Utc::now().timestamp();
    let clip = Clip {
        id: DbUuid::new(Uuid::new_v4()),
        user_id: DbUuid::new(user_id),
        account_id: DbUuid::new(body.account_id),
        message_id: body.message_id,
        snippet,
        created_at: now,
    };
    clip_repo(&state)
        .create(&clip)
        .await
        .map_err(KestrelError::from)?;
    Ok((StatusCode::CREATED, Json(clip)))
}

/// GET /api/v1/clips — newest first, all own accounts.
pub async fn list_clips(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<Clip>>, KestrelError> {
    let clips = clip_repo(&state)
        .list_by_user(user_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(clips))
}

/// DELETE /api/v1/clips/:id — owner only (scoped by user id).
pub async fn delete_clip(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(clip_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    let deleted = clip_repo(&state)
        .delete(clip_id, user_id)
        .await
        .map_err(KestrelError::from)?;
    if !deleted {
        return Err(KestrelError::NotFound("Clip not found".to_string()));
    }
    Ok(StatusCode::NO_CONTENT)
}

// --- Thread sticky notes (local-only, like subject overrides) ---

#[derive(Debug, Deserialize, specta::Type)]
pub struct SetNoteRequest {
    pub account_id: Uuid,
    pub note: String,
}

/// PUT /api/v1/threads/:thread_id/notes — create or replace the sticky note.
pub async fn set_note(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(thread_id): Path<String>,
    Json(body): Json<SetNoteRequest>,
) -> Result<StatusCode, KestrelError> {
    let note = body.note.trim().to_string();
    if note.is_empty() || note.chars().count() > 2000 {
        return Err(KestrelError::BadRequest(
            "note must be 1-2000 characters".to_string(),
        ));
    }
    validate_thread_id(&thread_id)?;
    verify_account_owner(&state, user_id, body.account_id).await?;
    note_repo(&state)
        .upsert(body.account_id, &thread_id, &note)
        .await
        .map_err(KestrelError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/threads/:thread_id/notes?account_id=... — remove the note.
pub async fn clear_note(
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
    note_repo(&state)
        .delete(account_id, &thread_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/threads/notes?account_id=... — map for display overlay.
pub async fn list_notes(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ThreadNote>>, KestrelError> {
    let account_id: Uuid = params
        .get("account_id")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| KestrelError::BadRequest("account_id is required".to_string()))?;
    verify_account_owner(&state, user_id, account_id).await?;
    let rows = note_repo(&state)
        .list_by_account(account_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(rows))
}
