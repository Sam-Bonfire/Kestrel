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

/// Cursor-based sync result for incremental message fetching.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncResult {
    pub messages: Vec<MessagePayload>,
    pub next_cursor: String,
}

/// Mail provider interface - implemented by each mail WASM plugin.
pub trait MailProvider: Send + Sync {
    /// Fetch new/updated message headers since the last cursor position.
    fn sync_mail(
        &self,
        auth_token: &str,
        cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError>;

    /// Fetch the full body of a single message by its provider-side external ID.
    fn fetch_message_body(
        &self,
        auth_token: &str,
        external_id: &str,
    ) -> Result<MessageBody, PluginError>;

    /// Soft-delete a message on the provider (moves to Trash).
    fn delete_message(
        &self,
        auth_token: &str,
        external_id: &str,
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

/// Attendee info.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Attendee {
    pub email: String,
    pub name: Option<String>,
    pub response_status: Option<String>,
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
pub trait CalendarProvider: Send + Sync {
    /// Fetch all calendars available under this account.
    fn fetch_calendars(
        &self,
        auth_token: &str,
    ) -> Result<Vec<CalendarPayload>, PluginError>;

    /// Fetch all events within a UTC timestamp range.
    fn fetch_events(
        &self,
        auth_token: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError>;

    /// Create or update a calendar event.
    fn mutate_event(
        &self,
        auth_token: &str,
        action: &str,
        payload: &EventPayload,
    ) -> Result<(), PluginError>;

    /// Soft-delete an event on the provider.
    fn delete_event(
        &self,
        auth_token: &str,
        external_id: &str,
    ) -> Result<(), PluginError>;
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
