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
    AuthUser { user_id }: AuthUser,
    Json(payload): Json<UpdateLabelRequest>,
) -> Result<Json<Label>, KestrelError> {
    if !super::calendars::verify_account_ownership(&state, user_id, payload.account_id).await? {
        return Err(KestrelError::NotFound("Account not found".to_string()));
    }

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
            repo.upsert(&label).await.map_err(KestrelError::Database)?;
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let repo = PostgresLabelRepository::new(pool.clone());
            repo.upsert(&label).await.map_err(KestrelError::Database)?;
        }
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
                state.jwt_secret.clone(),
            )
            .find_by_user_id(user_id)
            .await?
            .into_iter()
            .map(|a| a.id.0)
            .collect()
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )
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
