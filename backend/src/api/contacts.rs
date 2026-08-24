use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    api::router::AppState,
    core::repository::{AccountRepository, ContactRepository},
};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
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

    let (account_repo, contact_repo): (
        Box<dyn AccountRepository>,
        Box<dyn ContactRepository>,
    ) = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => (
            Box::new(crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )),
            Box::new(
                crate::db::sqlite::contact_repository::SqliteContactRepository::new(pool.clone()),
            ),
        ),
        crate::db::pool::DbPool::Postgres(pool) => (
            Box::new(crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )),
            Box::new(
                crate::db::postgres::contact_repository::PostgresContactRepository::new(pool.clone()),
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
