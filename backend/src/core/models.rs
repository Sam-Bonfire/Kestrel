use serde::{Deserialize, Serialize};

use super::types::DbUuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: DbUuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Label {
    pub id: DbUuid,
    pub account_id: DbUuid,
    pub name: String,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Account {
    pub id: DbUuid,
    pub user_id: DbUuid,
    pub provider: String,
    pub provider_account_id: String,
    pub display_name: String,
    #[serde(skip_serializing)]
    pub access_token: Option<String>,
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
    pub sync_error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: DbUuid,
    pub account_id: DbUuid,
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
    pub snoozed_until: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Calendar {
    pub id: DbUuid,
    pub account_id: DbUuid,
    pub external_id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CalendarEvent {
    pub id: DbUuid,
    pub account_id: DbUuid,
    pub calendar_id: DbUuid,
    pub external_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub is_all_day: bool,
    pub recurrence_rules: Option<String>,
    pub organizer_email: Option<String>,
    pub organizer_name: Option<String>,
    pub attendees: Option<String>,
    pub status: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OfflineQueueItem {
    pub id: i32,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub payload: Option<String>,
    pub queued_at: i64,
    pub retry_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attachment {
    pub id: DbUuid,
    pub message_id: DbUuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub external_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPreferences {
    pub user_id: DbUuid,
    pub preferences_json: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contact {
    pub id: DbUuid,
    pub account_id: DbUuid,
    pub name: Option<String>,
    pub email: String,
    pub avatar_url: Option<String>,
    pub last_contacted_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsPayload {
    pub mail_dense_mode: Option<bool>,
    pub mail_default_landing_view: Option<String>,
    pub mail_signature: Option<String>,
    pub label_customizations: Option<std::collections::HashMap<String, LabelCustomization>>,
    pub sync_interval: Option<i32>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LabelCustomization {
    pub icon_name: String,
    pub color_name: String,
}
