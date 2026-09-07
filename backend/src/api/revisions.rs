use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::HistoricalRevisionRepository;
use crate::db::pool::DbPool;
use crate::db::postgres::revision_repository::PostgresRevisionRepository;
use crate::db::sqlite::revision_repository::SqliteRevisionRepository;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Int,
    Bool,
    Text,
}

/// Whitelisted restorable columns per resource type: (table, [(column, type)]).
/// Anything outside this list in a revision payload is ignored.
fn restorable_columns(
    resource_type: &str,
) -> Option<(&'static str, &'static [(&'static str, ColumnType)])> {
    match resource_type {
        "message" => Some((
            "messages",
            &[
                ("subject", ColumnType::Text),
                ("sender_name", ColumnType::Text),
                ("sender_email", ColumnType::Text),
                ("snippet", ColumnType::Text),
                ("body_text", ColumnType::Text),
                ("body_html", ColumnType::Text),
                ("is_read", ColumnType::Bool),
                ("is_archived", ColumnType::Bool),
                ("is_deleted", ColumnType::Bool),
            ],
        )),
        "calendar_event" => Some((
            "calendar_events",
            &[
                ("title", ColumnType::Text),
                ("description", ColumnType::Text),
                ("location", ColumnType::Text),
                ("start_time", ColumnType::Int),
                ("end_time", ColumnType::Int),
                ("is_all_day", ColumnType::Bool),
                ("status", ColumnType::Text),
            ],
        )),
        _ => None,
    }
}

#[derive(Debug, Serialize)]
pub struct RestoreResponse {
    pub revision_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub restored_fields: Vec<String>,
    pub revision_number: i32,
}

fn coerce_value(value: &serde_json::Value, col_type: ColumnType) -> Option<RestorableValue> {
    match (value, col_type) {
        (serde_json::Value::Number(n), ColumnType::Int) => n.as_i64().map(RestorableValue::Int),
        (serde_json::Value::Bool(b), ColumnType::Bool) => Some(RestorableValue::Bool(*b)),
        (serde_json::Value::Number(n), ColumnType::Bool) => {
            n.as_i64().map(|v| RestorableValue::Bool(v != 0))
        }
        (serde_json::Value::String(s), ColumnType::Text) => Some(RestorableValue::Text(s.clone())),
        (serde_json::Value::Number(n), ColumnType::Text) => {
            Some(RestorableValue::Text(n.to_string()))
        }
        (serde_json::Value::Null, _) => None,
        _ => None,
    }
}

#[derive(Debug, Clone)]
enum RestorableValue {
    Int(i64),
    Bool(bool),
    Text(String),
}

pub async fn restore_revision(
    State(state): State<AppState>,
    AuthUser { user_id: _ }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<RestoreResponse>, KestrelError> {
    let revision_id = id
        .parse::<uuid::Uuid>()
        .map_err(|_| KestrelError::BadRequest("Invalid revision id".to_string()))?;

    let revision = match &state.db {
        DbPool::Sqlite(pool) => {
            SqliteRevisionRepository::new(pool.clone())
                .find_by_id(revision_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            PostgresRevisionRepository::new(pool.clone())
                .find_by_id(revision_id)
                .await?
        }
    };

    let revision =
        revision.ok_or_else(|| KestrelError::NotFound("Revision not found".to_string()))?;

    let (table, columns) = restorable_columns(&revision.resource_type).ok_or_else(|| {
        KestrelError::BadRequest(format!(
            "Cannot restore resource type '{}'",
            revision.resource_type
        ))
    })?;

    let payload: serde_json::Value = serde_json::from_str(&revision.serialized_payload)
        .map_err(|_| KestrelError::BadRequest("Revision payload is not valid JSON".to_string()))?;
    let payload = payload.as_object().ok_or_else(|| {
        KestrelError::BadRequest("Revision payload must be a JSON object".to_string())
    })?;

    let mut assignments: Vec<(String, RestorableValue)> = Vec::new();
    for (column, col_type) in columns {
        if let Some(value) = payload
            .get(*column)
            .and_then(|v| coerce_value(v, *col_type))
        {
            assignments.push(((*column).to_string(), value));
        }
    }

    if assignments.is_empty() {
        return Err(KestrelError::BadRequest(
            "Revision payload contains no restorable fields".to_string(),
        ));
    }

    let restored_fields: Vec<String> = assignments.iter().map(|(c, _)| c.clone()).collect();

    match &state.db {
        DbPool::Sqlite(pool) => {
            let placeholders: Vec<String> = assignments
                .iter()
                .map(|(c, _)| format!("{c} = ?"))
                .collect();
            let sql = format!(
                "UPDATE {table} SET {}, updated_at = unixepoch() WHERE id = ?",
                placeholders.join(", ")
            );
            let mut query = sqlx::query(&sql);
            for (_, value) in &assignments {
                query = match value {
                    // SQLite stores flags as INTEGER 0/1.
                    RestorableValue::Int(n) => query.bind(n),
                    RestorableValue::Bool(b) => query.bind(i64::from(*b)),
                    RestorableValue::Text(s) => query.bind(s),
                };
            }
            let result = query
                .bind(revision.resource_id.to_string())
                .execute(pool)
                .await?;
            if result.rows_affected() == 0 {
                return Err(KestrelError::NotFound("Resource not found".to_string()));
            }
        }
        DbPool::Postgres(pool) => {
            let placeholders: Vec<String> = assignments
                .iter()
                .enumerate()
                .map(|(i, (c, _))| format!("{c} = ${}", i + 1))
                .collect();
            let sql = format!(
                "UPDATE {table} SET {}, updated_at = EXTRACT(EPOCH FROM NOW())::BIGINT WHERE id = ${}",
                placeholders.join(", "),
                assignments.len() + 1
            );
            let mut query = sqlx::query(&sql);
            for (_, value) in &assignments {
                query = match value {
                    // Postgres has native BOOLEAN columns.
                    RestorableValue::Int(n) => query.bind(n),
                    RestorableValue::Bool(b) => query.bind(b),
                    RestorableValue::Text(s) => query.bind(s),
                };
            }
            let result = query.bind(revision.resource_id).execute(pool).await?;
            if result.rows_affected() == 0 {
                return Err(KestrelError::NotFound("Resource not found".to_string()));
            }
        }
    }

    Ok(Json(RestoreResponse {
        revision_id: revision.id.to_string(),
        resource_type: revision.resource_type,
        resource_id: revision.resource_id.to_string(),
        restored_fields,
        revision_number: revision.revision_number,
    }))
}
