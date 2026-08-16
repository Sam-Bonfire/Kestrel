use std::fmt;

// ─────────────────────────────────────────────
// Mail Provider Interface (mirrors WIT)
// ─────────────────────────────────────────────

/// A message header/snippet payload returned during sync.
/// NOTE: body-text and body-html are intentionally absent here.
/// Full bodies are fetched on-demand via fetch_message_body.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessagePayload {
    pub id: String,
    pub external_id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub sender_name: Option<String>,
    pub sender_email: String,
    pub recipients: String,
    pub date_sent: i64,
    pub date_received: i64,
    pub snippet: Option<String>,
    pub labels: Option<String>,
    pub is_read: bool,
}

/// Full message body fetched on-demand when a message is opened.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageBody {
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}

/// Payload for sending an email.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentPayload {
    pub filename: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

pub struct SendMessagePayload {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_html: String,
    pub attachments: Option<Vec<AttachmentPayload>>,
}

/// Cursor-based sync result for incremental message fetching.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncResult {
    pub messages: Vec<MessagePayload>,
    pub next_cursor: String,
}

use async_trait::async_trait;

/// Mail provider interface - implemented by each mail WASM plugin.
#[async_trait]
pub trait MailProvider: Send + Sync {
    /// Fetch new/updated message headers since the last cursor position.
    async fn sync_mail(
        &self,
        auth_token: &str,
        cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError>;

    /// Fetch the full body of a single message by its provider-side external ID.
    async fn fetch_message_body(
        &self,
        auth_token: &str,
        external_id: &str,
    ) -> Result<MessageBody, PluginError>;

    /// Download an attachment by its provider-side IDs.
    async fn download_attachment(
        &self,
        auth_token: &str,
        external_message_id: &str,
        external_attachment_id: &str,
    ) -> Result<Vec<u8>, PluginError>;

    /// Fetch or construct a direct CDN download URL for an attachment.
    async fn get_attachment_url(
        &self,
        _auth_token: &str,
        external_message_id: &str,
        external_attachment_id: &str,
    ) -> Result<String, PluginError> {
        Ok(format!(
            "https://cdn.kestrel.dev/attachments/{}/{}",
            external_message_id, external_attachment_id
        ))
    }

    /// Send an email message.
    async fn send_message(
        &self,
        auth_token: &str,
        payload: SendMessagePayload,
    ) -> Result<(), PluginError>;
}

// ─────────────────────────────────────────────
// Calendar Provider Interface (mirrors WIT)
// ─────────────────────────────────────────────

/// Calendar metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalendarPayload {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
}

/// Calendar event payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventPayload {
    pub id: String,
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
}

/// Calendar provider interface - implemented by each calendar WASM plugin.
#[async_trait]
pub trait CalendarProvider: Send + Sync {
    /// Fetch all calendars available under this account.
    async fn fetch_calendars(&self, auth_token: &str) -> Result<Vec<CalendarPayload>, PluginError>;

    /// Fetch all events within a UTC timestamp range.
    async fn fetch_events(
        &self,
        auth_token: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError>;

    /// Create or update a calendar event.
    async fn mutate_event(
        &self,
        auth_token: &str,
        action: &str,
        payload: &EventPayload,
    ) -> Result<(), PluginError>;

    /// Soft-delete an event on the provider.
    async fn delete_event(&self, auth_token: &str, external_id: &str) -> Result<(), PluginError>;
}

// ─────────────────────────────────────────────
// Provider Branding Interface (mirrors WIT)
// ─────────────────────────────────────────────

/// Branding metadata returned by each plugin so the host UI can render
/// provider-specific connect buttons and account badges dynamically.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrandingPayload {
    pub name: String,
    pub button_text: String,
    pub button_color: String,
    pub icon_svg: String,
}

/// Provider branding interface - implemented by each WASM plugin.
pub trait ProviderBranding: Send + Sync {
    /// Return the branding configuration for this provider.
    fn get_branding(&self) -> BrandingPayload;
}

// ─────────────────────────────────────────────
// Combined Plugin Trait
// ─────────────────────────────────────────────

/// Combined trait that a full-featured provider plugin must implement.
pub trait ProviderPlugin: ProviderBranding + MailProvider + CalendarProvider + Send + Sync {
    /// Unique identifier for this plugin (e.g. "gmail", "outlook").
    fn id(&self) -> &str;
}

// ─────────────────────────────────────────────
// Error Type
// ─────────────────────────────────────────────

/// Error type for plugin operations.
#[derive(Debug)]
pub struct PluginError(pub String);

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plugin error: {}", self.0)
    }
}

impl std::error::Error for PluginError {}

impl From<String> for PluginError {
    fn from(s: String) -> Self {
        PluginError(s)
    }
}

impl From<&str> for PluginError {
    fn from(s: &str) -> Self {
        PluginError(s.to_string())
    }
}
