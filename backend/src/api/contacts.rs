use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api::auth::AuthUser,
    api::router::AppState,
    core::error::KestrelError,
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

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateNotesRequest {
    pub account_id: String,
    pub email: String,
    pub notes: String,
}

pub async fn update_contact_notes(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateNotesRequest>,
) -> Result<Json<serde_json::Value>, KestrelError> {
    use std::str::FromStr;
    let account_id = Uuid::from_str(&body.account_id)
        .map_err(|_| KestrelError::BadRequest("Invalid account id".to_string()))?;
    if body.notes.chars().count() > 2000 {
        return Err(KestrelError::BadRequest(
            "Notes exceeds 2000 characters".to_string(),
        ));
    }

    let contact_repo: Box<dyn ContactRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::contact_repository::SqliteContactRepository::new(pool.clone()),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::contact_repository::PostgresContactRepository::new(pool.clone()),
        ),
    };

    // Ownership check: the account must belong to the caller.
    let account_repo: Box<dyn AccountRepository> = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            ),
        ),
        crate::db::pool::DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            ),
        ),
    };
    let owned = account_repo
        .find_by_user_id(user.user_id)
        .await?
        .into_iter()
        .any(|a| a.id.0 == account_id);
    if !owned {
        return Err(KestrelError::NotFound("Account not found".to_string()));
    }

    contact_repo
        .set_notes(account_id, &body.email, &body.notes)
        .await?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

fn user_account_ids(
    accounts: &[crate::core::models::Account],
    requested: Option<Uuid>,
) -> Result<Vec<Uuid>, (StatusCode, String)> {
    if let Some(id) = requested {
        if accounts.iter().any(|a| a.id.0 == id) {
            return Ok(vec![id]);
        }
        return Err((StatusCode::NOT_FOUND, "Account not found".to_string()));
    }
    Ok(accounts.iter().map(|a| a.id.0).collect())
}

async fn user_repos(state: &AppState) -> (Box<dyn AccountRepository>, Box<dyn ContactRepository>) {
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

fn csv_escape(field: &str) -> String {
    format!("\"{}\"", field.replace('"', "\"\""))
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct ExportQuery {
    pub account_id: Option<String>,
}

/// Download all contacts as CSV (name,email per row).
pub async fn export_contacts(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ExportQuery>,
) -> Result<Response, (StatusCode, String)> {
    use std::str::FromStr;
    let requested = query
        .account_id
        .map(|id| Uuid::from_str(&id))
        .transpose()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid account id".to_string()))?;

    let (account_repo, contact_repo) = user_repos(&state).await;
    let accounts = account_repo
        .find_by_user_id(user.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let account_ids = user_account_ids(&accounts, requested)?;

    let contacts = contact_repo
        .search(&account_ids, "", 100_000)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    // ponytail: full-table export into String; stream the response if contacts ever exceed ~100k.

    let mut csv = String::from("name,email\n");
    for c in contacts {
        csv.push_str(&format!(
            "{},{}\n",
            csv_escape(&c.name.unwrap_or_default()),
            csv_escape(&c.email)
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("text/csv"));
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_static("attachment; filename=\"kestrel-contacts.csv\""),
    );
    Ok((headers, csv).into_response())
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct ImportRequest {
    pub account_id: String,
    /// "csv" or "vcard"
    pub format: String,
    pub content: String,
}

#[derive(Debug, serde::Serialize, specta::Type)]
pub struct ImportResponse {
    pub imported: i64,
    pub skipped: i64,
}

/// Minimal vCard parser: unfolds continuations, takes FN and EMAIL per card.
fn parse_vcard(content: &str) -> Vec<(Option<String>, String)> {
    let unfolded = content.replace("\r\n ", "").replace('\r', "\n");
    let mut cards: Vec<Vec<String>> = vec![];
    let mut current: Vec<String> = vec![];
    for line in unfolded.lines() {
        let upper = line.to_uppercase();
        if upper.starts_with("BEGIN:VCARD") {
            current = vec![];
        } else if upper.starts_with("END:VCARD") {
            cards.push(std::mem::take(&mut current));
        } else {
            current.push(line.to_string());
        }
    }
    cards
        .into_iter()
        .filter_map(|lines| {
            let mut name: Option<String> = None;
            let mut email: Option<String> = None;
            for line in lines {
                let upper = line.to_uppercase();
                if upper.starts_with("FN:") {
                    name = Some(line[3..].trim().to_string());
                } else if upper.starts_with("EMAIL") {
                    let Some((_, value)) = line.split_once(':') else {
                        continue;
                    };
                    let candidate = value.trim().to_string();
                    if candidate.contains('@') && email.is_none() {
                        email = Some(candidate);
                    }
                }
            }
            email.map(|e| (name, e))
        })
        .collect()
}

fn parse_csv(content: &str) -> Vec<(Option<String>, String)> {
    let mut rows = vec![];
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Skip a header row.
        if i == 0 && line.to_lowercase().starts_with("name,") {
            continue;
        }
        let fields = split_csv_line(line);
        let (name, email) = match fields.as_slice() {
            [first, second, ..] => {
                if second.contains('@') {
                    (empty_to_none(first), second.clone())
                } else if first.contains('@') {
                    (None, first.clone())
                } else {
                    continue;
                }
            }
            [only] => {
                if only.contains('@') {
                    (None, only.clone())
                } else {
                    continue;
                }
            }
            [] => continue,
        };
        rows.push((name, email));
    }
    rows
}

fn empty_to_none(s: &str) -> Option<String> {
    let trimmed = s.trim().trim_matches('"').trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Bulk import contacts into one owned account.
pub async fn import_contacts(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, (StatusCode, String)> {
    use std::str::FromStr;
    let account_id = Uuid::from_str(&body.account_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid account id".to_string()))?;
    if body.content.len() > 1_000_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Import too large (1MB max)".to_string(),
        ));
    }

    let (account_repo, contact_repo) = user_repos(&state).await;
    let accounts = account_repo
        .find_by_user_id(user.user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let account_ids = user_account_ids(&accounts, Some(account_id))?;

    let parsed = match body.format.as_str() {
        "csv" => parse_csv(&body.content),
        "vcard" => parse_vcard(&body.content),
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Format must be csv or vcard".to_string(),
            ));
        }
    };

    let now = chrono::Utc::now().timestamp();
    let mut imported = 0i64;
    let mut skipped = 0i64;
    for (name, email) in parsed.into_iter().take(5000) {
        let contact = crate::core::models::Contact {
            id: uuid::Uuid::new_v4().into(),
            account_id: crate::core::types::DbUuid::from(account_ids[0]),
            name,
            email,
            avatar_url: None,
            notes: None,
            last_contacted_at: now,
            created_at: now,
        };
        match contact_repo.upsert(&contact).await {
            Ok(()) => imported += 1,
            Err(e) => {
                tracing::warn!("Contact import upsert failed: {}", e);
                skipped += 1;
            }
        }
    }

    Ok(Json(ImportResponse { imported, skipped }))
}

#[cfg(test)]
mod tests {
    use super::{parse_csv, parse_vcard};

    #[test]
    fn test_parse_csv_with_header_and_quotes() {
        let rows = parse_csv("name,email\n\"Doe, Jane\",jane@example.com\nbob@example.com\n");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0.as_deref(), Some("Doe, Jane"));
        assert_eq!(rows[0].1, "jane@example.com");
        assert_eq!(rows[1].0, None);
    }

    #[test]
    fn test_parse_vcard_unfolds_and_picks_email() {
        let vcard = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Jane Doe\r\nEMAIL;TYPE=HOME:jane@example.com\r\nEND:VCARD\r\nBEGIN:VCARD\r\nVERSION:3.0\r\nEMAIL:no-name@example.com\r\nEND:VCARD\r\n";
        let rows = parse_vcard(vcard);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0.as_deref(), Some("Jane Doe"));
        assert_eq!(rows[1].0, None);
    }

    #[test]
    fn test_parse_csv_skips_junk() {
        assert!(parse_csv("not an email line\n").is_empty());
        assert!(parse_csv("").is_empty());
    }
}
