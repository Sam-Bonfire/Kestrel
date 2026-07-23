use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::MessageRepository;
use crate::db::pool::DbPool;
use crate::db::sqlite::message_repository::SqliteMessageRepository;
use crate::db::postgres::message_repository::PostgresMessageRepository;

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_search_limit")]
    pub limit: i64,
}

fn default_search_limit() -> i64 {
    20
}

#[derive(Serialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: String,
    pub snippet: Option<String>,
    pub date_received: i64,
    pub is_read: bool,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
    pub query: String,
}

/// K-044: GET /api/v1/search?q=... — FTS5 full-text search across messages.
pub async fn search_messages(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, KestrelError> {
    if params.q.trim().is_empty() {
        return Err(KestrelError::BadRequest(
            "Search query cannot be empty".to_string(),
        ));
    }

    let limit = params.limit.min(50).max(1);

    let messages = match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.search(user_id, &params.q, limit).await?
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.search(user_id, &params.q, limit).await?
        }
    };

    let total = messages.len();
    let query = params.q;

    let results: Vec<SearchResult> = messages.into_iter().map(|m| SearchResult {
        id: m.id.0,
        account_id: m.account_id.0,
        external_id: m.external_id,
        thread_id: m.thread_id,
        subject: m.subject,
        sender_name: m.sender_name,
        sender_email: m.sender_email,
        snippet: m.snippet,
        date_received: m.date_received,
        is_read: m.is_read,
    }).collect();

    Ok(Json(SearchResponse {
        results,
        total,
        query,
    }))
}
