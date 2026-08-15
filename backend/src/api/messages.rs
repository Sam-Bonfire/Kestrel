use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::repository::{AccountRepository, FilterRepository, MessageRepository};
use crate::db::pool::DbPool;
use crate::db::postgres::message_repository::PostgresMessageRepository;
use crate::db::sqlite::message_repository::SqliteMessageRepository;

// --- Request / Response types ---

#[derive(Deserialize)]
pub struct MessageListParams {
    pub account_id: Option<Uuid>,
    pub folder: Option<String>,
    pub cursor: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageSummary>,
    pub next_cursor: Option<String>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct MessageSummary {
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
    pub is_archived: bool,
    pub has_attachments: bool,
    pub labels: Option<String>,
}

#[derive(Serialize)]
pub struct MessageDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: String,
    pub recipients: String,
    pub date_sent: i64,
    pub date_received: i64,
    pub snippet: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub labels: Option<String>,
    pub is_read: bool,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub has_attachments: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Deserialize)]
pub struct StarParams {
    pub is_starred: bool,
}

#[derive(Deserialize)]
pub struct LabelParams {
    pub labels: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkActionType {
    MarkRead,
    Archive,
    Trash,
    ToggleStar,
}

#[derive(Deserialize)]
pub struct BulkActionParams {
    pub message_ids: Vec<Uuid>,
    pub action: BulkActionType,
    // Provide an action_value, e.g. boolean for mark_read
    pub action_value: Option<bool>,
}

#[derive(Deserialize)]
pub struct ThreadParams {
    pub thread_id: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

// --- K-039: GET /api/v1/messages ---

pub async fn list_messages(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Query(params): Query<MessageListParams>,
) -> Result<Json<MessageListResponse>, KestrelError> {
    let limit = params.limit.min(50).max(1);
    let folder = params.folder.as_deref();
    let cursor = params.cursor.as_deref();

    let messages =
        list_messages_from_db(&state, user_id, params.account_id, folder, cursor, limit).await?;

    let next_cursor = if messages.len() == limit as usize {
        messages.last().map(|m| m.date_received.to_string())
    } else {
        None
    };

    let total = messages.len();

    let summaries: Vec<MessageSummary> = messages
        .into_iter()
        .map(|m| MessageSummary {
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
            is_archived: m.is_archived,
            has_attachments: m.has_attachments,
            labels: m.labels,
        })
        .collect();

    Ok(Json(MessageListResponse {
        messages: summaries,
        next_cursor,
        total,
    }))
}

async fn list_messages_from_db(
    state: &AppState,
    user_id: Uuid,
    account_id: Option<Uuid>,
    folder: Option<&str>,
    cursor: Option<&str>,
    limit: i64,
) -> Result<Vec<crate::core::models::Message>, KestrelError> {
    // If account_id is provided, verify it belongs to the user
    let account_id = match account_id {
        Some(aid) => {
            // Verify ownership
            let owns = verify_account_ownership(state, user_id, aid).await?;
            if !owns {
                return Err(KestrelError::NotFound("Account not found".to_string()));
            }
            Some(aid)
        }
        None => {
            // No account filter — list from all user accounts
            // For simplicity, we pass None and let the repo handle it
            None
        }
    };

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            Ok(repo.list(account_id, folder, cursor, limit).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            Ok(repo.list(account_id, folder, cursor, limit).await?)
        }
    }
}

// --- K-040: GET /api/v1/messages/:id ---

pub async fn get_message(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<Json<MessageDetail>, KestrelError> {
    let mut msg = find_message_from_db(&state, message_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;

    // Verify the message belongs to an account owned by this user
    let owns = verify_account_ownership(&state, user_id, msg.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Message not found".to_string()));
    }

    // On-demand body fetching (Task 2.2)
    if msg.body_text.is_none() && msg.body_html.is_none() {
        if let Some(account) = find_account_from_db(&state, msg.account_id.0).await? {
            if let Some(token) = &account.access_token {
                let plugin_manager = state.plugin_manager.read().await;
                if let Some(plugin) = plugin_manager.find_by_id(&account.provider) {
                    if let Ok(body) = plugin
                        .as_mail_provider()
                        .fetch_message_body(token, &msg.external_id)
                        .await
                    {
                        msg.body_text = body.body_text;
                        msg.body_html = body.body_html;
                        // Save back to DB
                        match &state.db {
                            DbPool::Sqlite(pool) => {
                                let repo = SqliteMessageRepository::new(pool.clone());
                                let _ = repo.upsert(&msg).await;
                            }
                            DbPool::Postgres(pool) => {
                                let repo = PostgresMessageRepository::new(pool.clone());
                                let _ = repo.upsert(&msg).await;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(Json(MessageDetail {
        id: msg.id.0,
        account_id: msg.account_id.0,
        external_id: msg.external_id,
        thread_id: msg.thread_id,
        subject: msg.subject,
        sender_name: msg.sender_name,
        sender_email: msg.sender_email,
        recipients: msg.recipients,
        date_sent: msg.date_sent,
        date_received: msg.date_received,
        snippet: msg.snippet,
        body_text: msg.body_text,
        body_html: msg.body_html,
        labels: msg.labels,
        is_read: msg.is_read,
        is_archived: msg.is_archived,
        is_deleted: msg.is_deleted,
        has_attachments: msg.has_attachments,
        created_at: msg.created_at,
        updated_at: msg.updated_at,
    }))
}

async fn find_message_from_db(
    state: &AppState,
    message_id: Uuid,
) -> Result<Option<crate::core::models::Message>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            Ok(repo.find_by_id(message_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            Ok(repo.find_by_id(message_id).await?)
        }
    }
}

async fn find_account_from_db(
    state: &AppState,
    account_id: Uuid,
) -> Result<Option<crate::core::models::Account>, KestrelError> {
    use crate::core::repository::AccountRepository;
    use crate::db::postgres::account_repository::PostgresAccountRepository;
    use crate::db::sqlite::account_repository::SqliteAccountRepository;

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteAccountRepository::new(pool.clone());
            Ok(repo.find_by_id(account_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone());
            Ok(repo.find_by_id(account_id).await?)
        }
    }
}

// --- K-041: POST /api/v1/messages/:id/read ---

pub async fn mark_read(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;
    set_message_read(&state, message_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_message_read(
    state: &AppState,
    message_id: Uuid,
    is_read: bool,
) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_read(message_id, is_read).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_read(message_id, is_read).await?;
        }
    }
    Ok(())
}

// --- K-042: POST /api/v1/messages/:id/archive ---

pub async fn archive_message(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;
    set_message_archived(&state, message_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn snooze_message(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    // Basic implementation for snooze
    verify_message_ownership(&state, user_id, message_id).await?;
    set_message_archived(&state, message_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_message_archived(
    state: &AppState,
    message_id: Uuid,
    is_archived: bool,
) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_archived(message_id, is_archived).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_archived(message_id, is_archived).await?;
        }
    }
    Ok(())
}

// --- K-043: POST /api/v1/messages/:id/trash ---

pub async fn trash_message(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;
    set_message_deleted(&state, message_id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_message_deleted(
    state: &AppState,
    message_id: Uuid,
    is_deleted: bool,
) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_deleted(message_id, is_deleted).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_deleted(message_id, is_deleted).await?;
        }
    }
    Ok(())
}

// --- Ownership verification helpers ---

async fn verify_account_ownership(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<bool, KestrelError> {
    use crate::core::repository::AccountRepository;
    use crate::db::postgres::account_repository::PostgresAccountRepository;
    use crate::db::sqlite::account_repository::SqliteAccountRepository;

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteAccountRepository::new(pool.clone());
            let account = repo.find_by_id(account_id).await?;
            Ok(account.map(|a| *a.user_id == user_id).unwrap_or(false))
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone());
            let account = repo.find_by_id(account_id).await?;
            Ok(account.map(|a| *a.user_id == user_id).unwrap_or(false))
        }
    }
}

async fn verify_message_ownership(
    state: &AppState,
    user_id: Uuid,
    message_id: Uuid,
) -> Result<(), KestrelError> {
    let msg = find_message_from_db(state, message_id).await?;
    let msg = msg.ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;
    let owns = verify_account_ownership(state, user_id, msg.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Message not found".to_string()));
    }
    Ok(())
}

// --- Toggle Star (via Labels) ---

pub async fn toggle_star(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
    axum::Json(params): axum::Json<StarParams>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;

    let msg = find_message_from_db(&state, message_id).await?.unwrap();
    let mut labels: Vec<String> = msg
        .labels
        .and_then(|l| serde_json::from_str(&l).ok())
        .unwrap_or_default();

    let star_label = "STARRED".to_string();

    if params.is_starred && !labels.contains(&star_label) {
        labels.push(star_label);
    } else if !params.is_starred {
        labels.retain(|l| l != "STARRED");
    }

    let labels_json = serde_json::to_string(&labels).ok();

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_labels(message_id, labels_json).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_labels(message_id, labels_json).await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- Update Labels ---

pub async fn update_labels(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
    axum::Json(params): axum::Json<LabelParams>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;

    let labels_json = serde_json::to_string(&params.labels).ok();

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_labels(message_id, labels_json).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_labels(message_id, labels_json).await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- Bulk Action ---

pub async fn bulk_action(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    axum::Json(params): axum::Json<BulkActionParams>,
) -> Result<StatusCode, KestrelError> {
    for msg_id in &params.message_ids {
        if verify_message_ownership(&state, user_id, *msg_id)
            .await
            .is_err()
        {
            continue; // Skip messages they don't own
        }

        match params.action {
            BulkActionType::MarkRead => {
                let val = params.action_value.unwrap_or(true);
                set_message_read(&state, *msg_id, val).await?;
            }
            BulkActionType::Archive => {
                let val = params.action_value.unwrap_or(true);
                set_message_archived(&state, *msg_id, val).await?;
            }
            BulkActionType::Trash => {
                let val = params.action_value.unwrap_or(true);
                set_message_deleted(&state, *msg_id, val).await?;
            }
            BulkActionType::ToggleStar => {
                let val = params.action_value.unwrap_or(true);
                let msg = find_message_from_db(&state, *msg_id).await?.unwrap();
                let mut labels: Vec<String> = msg
                    .labels
                    .and_then(|l| serde_json::from_str(&l).ok())
                    .unwrap_or_default();

                let star_label = "STARRED".to_string();
                if val && !labels.contains(&star_label) {
                    labels.push(star_label);
                } else if !val {
                    labels.retain(|l| l != "STARRED");
                }

                let labels_json = serde_json::to_string(&labels).ok();

                match &state.db {
                    DbPool::Sqlite(pool) => {
                        let repo = SqliteMessageRepository::new(pool.clone());
                        repo.set_labels(*msg_id, labels_json).await?;
                    }
                    DbPool::Postgres(pool) => {
                        let repo = PostgresMessageRepository::new(pool.clone());
                        repo.set_labels(*msg_id, labels_json).await?;
                    }
                }
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

// --- Download Attachment (Task 3) ---

pub async fn download_attachment(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path((message_id, filename)): Path<(Uuid, String)>,
) -> Result<axum::response::Response, KestrelError> {
    // 1. Verify ownership and fetch message
    let msg = find_message_from_db(&state, message_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;
    verify_account_ownership(&state, user_id, msg.account_id.0).await?;

    // 2. Fetch the attachment metadata from DB
    let attachment_repo =
        crate::db::sqlite::attachment_repository::AttachmentRepository::new(match &state.db {
            DbPool::Sqlite(pool) => std::sync::Arc::new(pool.clone()),
            _ => {
                return Err(KestrelError::Internal(Box::new(
                    crate::core::error::SimpleError(
                        "Postgres not implemented for attachments".into(),
                    ),
                )));
            }
        });

    let attachments = attachment_repo
        .get_attachments_for_message(crate::core::types::DbUuid(message_id))
        .await?;
    let attachment = attachments
        .iter()
        .find(|a| a.filename == filename)
        .ok_or_else(|| KestrelError::NotFound("Attachment not found".into()))?;

    let external_attachment_id = attachment
        .external_id
        .as_ref()
        .ok_or_else(|| KestrelError::BadRequest("Attachment has no external ID".into()))?;

    // 3. Fetch account and tokens
    let account = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .find_by_id(msg.account_id.0)
                .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
                .find_by_id(msg.account_id.0)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    // 4. Download via plugin
    let plugin_manager = state.plugin_manager.read().await;
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    let bytes = plugin
        .as_mail_provider()
        .download_attachment(&auth_token, &msg.external_id, external_attachment_id)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin error: {:?}",
                e
            ))))
        })?;

    // 5. Stream to client
    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            attachment.content_type.clone(),
        )
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", attachment.filename),
        )
        .body(axum::body::Body::from(bytes))
        .unwrap())
}

// --- Redirect Attachment (Task K-122) ---

pub async fn redirect_attachment(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path((message_id, filename)): Path<(Uuid, String)>,
) -> Result<axum::response::Response, KestrelError> {
    // 1. Fetch message from DB
    let msg = find_message_from_db(&state, message_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;

    // 2. Verify account ownership (return 403 Forbidden if not owned)
    let owns = verify_account_ownership(&state, user_id, msg.account_id.0).await?;
    if !owns {
        return Err(KestrelError::Forbidden(
            "You do not own this account".to_string(),
        ));
    }

    // 3. Fetch attachment metadata from DB
    let attachment_repo =
        crate::db::sqlite::attachment_repository::AttachmentRepository::new(match &state.db {
            DbPool::Sqlite(pool) => std::sync::Arc::new(pool.clone()),
            _ => {
                return Err(KestrelError::Internal(Box::new(
                    crate::core::error::SimpleError(
                        "Postgres not implemented for attachments".into(),
                    ),
                )));
            }
        });

    let attachments = attachment_repo
        .get_attachments_for_message(crate::core::types::DbUuid(message_id))
        .await?;

    let attachment = attachments
        .iter()
        .find(|a| a.filename == filename)
        .ok_or_else(|| KestrelError::NotFound("Attachment not found".into()))?;

    let external_attachment_id = attachment
        .external_id
        .as_ref()
        .ok_or_else(|| KestrelError::BadRequest("Attachment has no external ID".into()))?;

    // 4. Fetch account details
    let account = find_account_from_db(&state, msg.account_id.0)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    // 5. Query plugin for CDN URL
    let plugin_manager = state.plugin_manager.read().await;
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    let cdn_url = plugin
        .as_mail_provider()
        .get_attachment_url(&auth_token, &msg.external_id, external_attachment_id)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin error: {:?}",
                e
            ))))
        })?;

    // 6. Return 302 Redirect with Location header
    Ok(axum::response::Response::builder()
        .status(StatusCode::FOUND)
        .header(axum::http::header::LOCATION, cdn_url)
        .body(axum::body::Body::empty())
        .unwrap())
}

// --- Outbound Send Logic (Task 37) ---

#[derive(serde::Deserialize)]
pub struct SendAttachmentPayload {
    pub filename: String,
    pub content_type: String,
    pub base64_content: String,
}

#[derive(serde::Deserialize)]
pub struct SendMessageRequest {
    pub account_id: Uuid,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_html: String,
    pub attachments: Option<Vec<SendAttachmentPayload>>,
}

#[derive(Serialize)]
pub struct SendMessageResponse {
    pub id: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    axum::Json(payload): axum::Json<SendMessageRequest>,
) -> Result<Json<SendMessageResponse>, KestrelError> {
    // 1. Fetch account and verify ownership
    let account = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .find_by_id(payload.account_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
                .find_by_id(payload.account_id)
                .await?
        }
    };

    let account = account.ok_or_else(|| KestrelError::NotFound("Account not found".to_string()))?;
    if *account.user_id != user_id {
        return Err(KestrelError::NotFound("Account not found".to_string()));
    }

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    // 2. Map request payload to plugin WIT payload
    use base64::{Engine as _, engine::general_purpose::STANDARD as b64};
    let mapped_attachments = match payload.attachments {
        Some(atts) => {
            let mut mapped = Vec::new();
            for a in atts {
                // Remove data uri prefix if present
                let b64_str = if a.base64_content.contains(",") {
                    a.base64_content
                        .split(",")
                        .nth(1)
                        .unwrap_or(&a.base64_content)
                } else {
                    &a.base64_content
                };
                let bytes = b64.decode(b64_str).map_err(|e| {
                    KestrelError::BadRequest(format!("Invalid base64 attachment: {}", e))
                })?;
                mapped.push(crate::plugins::traits::AttachmentPayload {
                    filename: a.filename,
                    content_type: a.content_type,
                    content: bytes,
                });
            }
            Some(mapped)
        }
        None => None,
    };

    let wit_payload = crate::plugins::traits::SendMessagePayload {
        to: payload.to,
        cc: payload.cc,
        bcc: payload.bcc,
        subject: payload.subject,
        body_html: payload.body_html,
        attachments: mapped_attachments,
    };

    // 3. Dispatch to plugin manager
    let plugin_manager = state.plugin_manager.read().await;
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    plugin
        .as_mail_provider()
        .send_message(&auth_token, wit_payload)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Dispatch failed: {:?}",
                e
            ))))
        })?;

    // 4. Return success response (UI expects SendMessageResponse)
    Ok(Json(SendMessageResponse {
        id: format!("sent-{}", uuid::Uuid::new_v4()),
    }))
}

pub async fn mute_thread(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    let msg = find_message_from_db(&state, message_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;

    let owns = verify_account_ownership(&state, user_id, msg.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Message not found".to_string()));
    }

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.set_thread_muted(&msg.thread_id).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.set_thread_muted(&msg.thread_id).await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn report_phishing(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    verify_message_ownership(&state, user_id, message_id).await?;
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteMessageRepository::new(pool.clone());
            repo.report_phishing(message_id).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresMessageRepository::new(pool.clone());
            repo.report_phishing(message_id).await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_raw_eml(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(message_id): Path<Uuid>,
) -> Result<axum::response::Response, KestrelError> {
    let msg = find_message_from_db(&state, message_id)
        .await?
        .ok_or_else(|| KestrelError::NotFound("Message not found".to_string()))?;

    let owns = verify_account_ownership(&state, user_id, msg.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Message not found".to_string()));
    }

    let eml_content = format!(
        "To: {}\nFrom: {}\nSubject: {}\n\n{}",
        msg.recipients,
        msg.sender_email,
        msg.subject.unwrap_or_default(),
        msg.body_text.unwrap_or_default()
    );

    Ok(axum::response::Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "message/rfc822")
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"message-{}.eml\"", message_id),
        )
        .body(axum::body::Body::from(eml_content))
        .unwrap())
}

#[derive(serde::Deserialize)]
pub struct BlockSenderRequest {
    pub email: String,
}

pub async fn block_sender(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    axum::Json(payload): axum::Json<BlockSenderRequest>,
) -> Result<StatusCode, KestrelError> {
    tracing::info!("User {} blocked sender {}", user_id, payload.email);

    match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            let filter_repo =
                crate::db::sqlite::filter_repository::SqliteFilterRepository::new(pool.clone());
            filter_repo.block_sender(user_id, &payload.email).await?;
            let msg_repo =
                crate::db::sqlite::message_repository::SqliteMessageRepository::new(pool.clone());
            msg_repo.trash_by_sender(user_id, &payload.email).await?;
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            let filter_repo =
                crate::db::postgres::filter_repository::PostgresFilterRepository::new(pool.clone());
            filter_repo.block_sender(user_id, &payload.email).await?;
            let msg_repo = crate::db::postgres::message_repository::PostgresMessageRepository::new(
                pool.clone(),
            );
            msg_repo.trash_by_sender(user_id, &payload.email).await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
