use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    api::router::AppState,
    core::repository::{AccountRepository, ContactRepository},
};

#[derive(Debug, Deserialize, specta::Type)]
pub struct SearchQuery {
    pub q: String,
    #[specta(type = Option<f64>)]
    pub limit: Option<i64>,
}

pub async fn search_contacts(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<crate::core::models::Contact>>, (axum::http::StatusCode, String)> {
    if query.q.is_empty() {
        return Ok(Json(vec![]));
    }

    let limit = query.limit.unwrap_or(10).clamp(1, 50);

    let (account_repo, contact_repo): (Box<dyn AccountRepository>, Box<dyn ContactRepository>) =
        match &state.db {
            crate::db::pool::DbPool::Sqlite(pool) => (
                Box::new(
                    crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                        pool.clone(),
                        state.jwt_secret.clone(),
                    ),
                ),
                Box::new(
                    crate::db::sqlite::contact_repository::SqliteContactRepository::new(
                        pool.clone(),
                    ),
                ),
            ),
            crate::db::pool::DbPool::Postgres(pool) => (
                Box::new(
                    crate::db::postgres::account_repository::PostgresAccountRepository::new(
                        pool.clone(),
                        state.jwt_secret.clone(),
                    ),
                ),
                Box::new(
                    crate::db::postgres::contact_repository::PostgresContactRepository::new(
                        pool.clone(),
                    ),
                ),
            ),
        };

    let accounts = account_repo
        .find_by_user_id(user.user_id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let account_ids: Vec<Uuid> = accounts.into_iter().map(|a| a.id.0).collect();

    let contacts = contact_repo
        .search(&account_ids, &query.q, limit)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(contacts))
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct ListQuery {
    pub account_id: Option<String>,
    #[specta(type = Option<f64>)]
    pub limit: Option<i64>,
}

type ApiError = (axum::http::StatusCode, String);

fn repos(state: &AppState) -> (Box<dyn AccountRepository>, Box<dyn ContactRepository>) {
    match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => (
            Box::new(
                crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                    pool.clone(),
                    state.jwt_secret.clone(),
                ),
            ),
            Box::new(
                crate::db::sqlite::contact_repository::SqliteContactRepository::new(pool.clone()),
            ),
        ),
        crate::db::pool::DbPool::Postgres(pool) => (
            Box::new(
                crate::db::postgres::account_repository::PostgresAccountRepository::new(
                    pool.clone(),
                    state.jwt_secret.clone(),
                ),
            ),
            Box::new(
                crate::db::postgres::contact_repository::PostgresContactRepository::new(
                    pool.clone(),
                ),
            ),
        ),
    }
}

async fn owned_account_ids(
    account_repo: &dyn AccountRepository,
    user_id: uuid::Uuid,
    requested: Option<Uuid>,
) -> Result<Vec<Uuid>, ApiError> {
    let accounts = account_repo
        .find_by_user_id(user_id)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    match requested {
        Some(id) if accounts.iter().any(|a| a.id.0 == id) => Ok(vec![id]),
        Some(_) => Err((
            axum::http::StatusCode::NOT_FOUND,
            "Account not found".to_string(),
        )),
        None => Ok(accounts.into_iter().map(|a| a.id.0).collect()),
    }
}

/// List contacts for merge/deduplication tooling.
pub async fn list_contacts(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<crate::core::models::Contact>>, ApiError> {
    use std::str::FromStr;
    let requested = query
        .account_id
        .map(|id| Uuid::from_str(&id))
        .transpose()
        .map_err(|_| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid account id".to_string(),
            )
        })?;
    let limit = query.limit.unwrap_or(2000).clamp(1, 2000);

    let (account_repo, contact_repo) = repos(&state);
    let account_ids = owned_account_ids(account_repo.as_ref(), user.user_id, requested).await?;

    let contacts = contact_repo
        .search(&account_ids, "", limit)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(contacts))
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct DeleteContactRequest {
    pub account_id: String,
    pub email: String,
    /// Survivor address; recorded as a tombstone so sync won't resurrect the loser.
    pub keep_email: Option<String>,
}

/// Merge (or plain-delete) one contact from an owned account.
pub async fn delete_contact(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<DeleteContactRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use std::str::FromStr;
    let account_id = Uuid::from_str(&body.account_id).map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            "Invalid account id".to_string(),
        )
    })?;

    let (account_repo, contact_repo) = repos(&state);
    owned_account_ids(account_repo.as_ref(), user.user_id, Some(account_id)).await?;

    let merged = match body.keep_email {
        Some(keep) => {
            contact_repo
                .record_merge(account_id, &keep, &body.email)
                .await
        }
        None => contact_repo.delete(account_id, &body.email).await,
    }
    .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if !merged {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            "Contact not found".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}
