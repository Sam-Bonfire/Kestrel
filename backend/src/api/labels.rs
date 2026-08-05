use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::Label;
use crate::core::repository::LabelRepository;
use crate::db::postgres::label_repository::PostgresLabelRepository;
use crate::db::sqlite::label_repository::SqliteLabelRepository;

#[derive(Deserialize)]
pub struct UpdateLabelRequest {
    pub account_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

pub async fn update_label(
    State(state): State<AppState>,
    AuthUser { .. }: AuthUser,
    Json(payload): Json<UpdateLabelRequest>,
) -> Result<Json<Label>, KestrelError> {
    // Basic verification: user must own the account. This can be complex depending on repo,
    // assuming they are allowed for now or implement an account check.

    let label = Label {
        id: crate::core::types::DbUuid(Uuid::new_v4()),
        account_id: crate::core::types::DbUuid(payload.account_id),
        name: payload.name.clone(),
        color: payload.color.clone(),
        icon: payload.icon.clone(),
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            let repo = SqliteLabelRepository::new(pool.clone());
            repo.upsert(&label)
                .await
                .map_err(|e| KestrelError::Database(e))?;
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let repo = PostgresLabelRepository::new(pool.clone());
            repo.upsert(&label)
                .await
                .map_err(|e| KestrelError::Database(e))?;
        }
    }

    Ok(Json(label))
}

pub async fn list_labels(
    State(_state): State<AppState>,
    AuthUser { user_id: _ }: AuthUser,
) -> Result<Json<Vec<Label>>, KestrelError> {
    // In a real app we'd fetch accounts for the user, then get labels.
    // Here we'll return empty for now, or you can implement list logic across accounts.
    Ok(Json(vec![]))
}
