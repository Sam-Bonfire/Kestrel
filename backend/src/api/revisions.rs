use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::api::auth::AuthUser;
use crate::api::messages::verify_account_ownership;
use crate::api::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::{CalendarEvent, Message};
use crate::core::repository::{EventRepository, HistoricalRevisionRepository, MessageRepository};

#[allow(dead_code)]
pub async fn restore_revision(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, KestrelError> {
    let revision_repo: Box<dyn HistoricalRevisionRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::revision_repository::SqliteRevisionRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::revision_repository::PostgresRevisionRepository::new(pool.clone()),
        ),
    };

    let revision = match revision_repo.find_by_id(id).await? {
        Some(r) => r,
        None => {
            return Err(KestrelError::NotFound("Revision not found".to_string()));
        }
    };

    if revision.resource_type == "message" {
        let mut msg: Message = serde_json::from_str(&revision.serialized_payload)
            .map_err(|e| KestrelError::Internal(format!("Failed to deserialize: {}", e).into()))?;

        verify_account_ownership(&state, user_id, msg.account_id.0).await?;

        msg.has_conflict = false;
        msg.updated_at = chrono::Utc::now().timestamp();

        let msg_repo: Box<dyn MessageRepository> = match &state.db {
            crate::db::pool::DbPool::Sqlite(pool) => Box::new(
                crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone()),
            ),
            crate::db::pool::DbPool::Postgres(pool) => Box::new(
                crate::db::postgres::message_repository::PostgresMessageRepository::new(
                    pool.clone(),
                ),
            ),
        };

        msg_repo.upsert(&msg).await?;
        Ok(Json(serde_json::to_value(&msg).unwrap()))
    } else if revision.resource_type == "calendar_event" {
        let mut event: CalendarEvent = serde_json::from_str(&revision.serialized_payload)
            .map_err(|e| KestrelError::Internal(format!("Failed to deserialize: {}", e).into()))?;

        verify_account_ownership(&state, user_id, event.account_id.0).await?;

        event.has_conflict = false;
        event.updated_at = chrono::Utc::now().timestamp();

        let event_repo: Box<dyn EventRepository> = match &state.db {
            crate::db::pool::DbPool::Sqlite(pool) => Box::new(
                crate::db::sqlite::event_repository::SqliteEventRepository::new(pool.clone()),
            ),
            crate::db::pool::DbPool::Postgres(pool) => Box::new(
                crate::db::postgres::event_repository::PostgresEventRepository::new(pool.clone()),
            ),
        };

        event_repo.upsert(&event).await?;
        Ok(Json(serde_json::to_value(&event).unwrap()))
    } else {
        Err(KestrelError::Internal(
            format!("Unknown resource type: {}", revision.resource_type).into(),
        ))
    }
}
