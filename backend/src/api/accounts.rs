use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::AccountRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::account_repository::PostgresAccountRepository;
use crate::db::sqlite::account_repository::SqliteAccountRepository;

pub async fn list_accounts(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<crate::core::models::Account>>, KestrelError> {
    let accounts = match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteAccountRepository::new(pool.clone(), state.jwt_secret.clone())
                .find_by_user_id(user_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            PostgresAccountRepository::new(pool.clone(), state.jwt_secret.clone())
                .find_by_user_id(user_id)
                .await?
        }
    };
    Ok(Json(accounts))
}

pub async fn delete_account(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(account_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    let account = find_account(&state, account_id).await?;
    let account = account.ok_or_else(|| KestrelError::NotFound("Account not found".to_string()))?;

    if *account.user_id != user_id {
        return Err(KestrelError::NotFound("Account not found".to_string()));
    }

    delete_account_cascade(&state, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn find_account(
    state: &AppState,
    account_id: Uuid,
) -> Result<Option<crate::core::models::Account>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => Ok(SqliteAccountRepository::new(pool.clone(), state.jwt_secret.clone())
            .find_by_id(account_id)
            .await?),
        DbPool::Postgres(pool) => Ok(PostgresAccountRepository::new(pool.clone(), state.jwt_secret.clone())
            .find_by_id(account_id)
            .await?),
    }
}

async fn delete_account_cascade(state: &AppState, account_id: Uuid) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteAccountRepository::new(pool.clone(), state.jwt_secret.clone())
                .delete(account_id)
                .await?;
        }
        DbPool::Postgres(pool) => {
            PostgresAccountRepository::new(pool.clone(), state.jwt_secret.clone())
                .delete(account_id)
                .await?;
        }
    }
    Ok(())
}
