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
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
}

pub async fn update_label(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<UpdateLabelRequest>,
) -> Result<Json<Label>, KestrelError> {
    let mut label = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            SqliteLabelRepository::new(pool.clone()).find_by_id(payload.id).await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            PostgresLabelRepository::new(pool.clone()).find_by_id(payload.id).await?
        }
    }.ok_or_else(|| KestrelError::NotFound("Label not found".to_string()))?;

    if !super::calendars::verify_account_ownership(&state, user_id, label.account_id.0).await? {
        return Err(KestrelError::NotFound("Label not found".to_string()));
    }

    label.name = payload.name.clone();
    label.color = payload.color.clone();
    label.icon = payload.icon.clone();
    label.updated_at = chrono::Utc::now().timestamp();

    let rows_affected = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            SqliteLabelRepository::new(pool.clone()).update(&label).await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            PostgresLabelRepository::new(pool.clone()).update(&label).await?
        }
    };

    if rows_affected == 0 {
        return Err(KestrelError::NotFound("Label not found".to_string()));
    }

    Ok(Json(label))
}

pub async fn list_labels(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<Label>>, KestrelError> {
    use crate::core::repository::AccountRepository;
    use crate::db::pool::DbPool;

    let account_ids: Vec<Uuid> = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_user_id(user_id)
            .await?
            .into_iter()
            .map(|a| a.id.0)
            .collect()
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_user_id(user_id)
            .await?
            .into_iter()
            .map(|a| a.id.0)
            .collect()
        }
    };

    let labels = match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteLabelRepository::new(pool.clone())
                .list_by_accounts(&account_ids)
                .await?
        }
        DbPool::Postgres(pool) => {
            PostgresLabelRepository::new(pool.clone())
                .list_by_accounts(&account_ids)
                .await?
        }
    };

    Ok(Json(labels))
}
