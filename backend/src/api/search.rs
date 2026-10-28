use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::MessageRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::message_repository::PostgresMessageRepository;
use crate::db::sqlite::message_repository::SqliteMessageRepository;

#[derive(Deserialize, specta::Type)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_search_limit")]
    #[specta(type = f64)]
    pub limit: i64,
}

fn default_search_limit() -> i64 {
    20
}

/// FTS5 query syntax problems are client errors, not server failures.
fn search_error(e: sqlx::Error) -> KestrelError {
    if let sqlx::Error::Database(db) = &e {
        let message = db.message();
        if message.contains("fts5:")
            || message.contains("unterminated string")
            || message.contains("no such column")
            || message.contains("unknown special query")
        {
            return KestrelError::BadRequest("Invalid search query".to_string());
        }
    }
    KestrelError::Database(e)
}

#[derive(Serialize, specta::Type)]
pub struct SearchResult {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: String,
    pub snippet: Option<String>,
    #[specta(type = f64)]
    pub date_received: i64,
    pub is_read: bool,
    pub is_set_aside: bool,
}

#[derive(Serialize, specta::Type)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[specta(type = f64)]
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

    // shortcut: length bound only, deep FTS5 complexity limits if abuse shows up
    if params.q.len() > 256 {
        return Err(KestrelError::BadRequest(
            "Search query is too long".to_string(),
        ));
    }

    let limit = params.limit.clamp(1, 50);

    let messages = match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.search(user_id, &params.q, limit)
                .await
                .map_err(search_error)?
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.search(user_id, &params.q, limit)
                .await
                .map_err(search_error)?
        }
    };

    let total = messages.len();
    let query = params.q;

    let results: Vec<SearchResult> = messages
        .into_iter()
        .map(|m| SearchResult {
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
            is_set_aside: m.is_set_aside,
        })
        .collect();

    Ok(Json(SearchResponse {
        results,
        total,
        query,
    }))
}
