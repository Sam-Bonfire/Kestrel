use async_trait::async_trait;

use super::traits::{
    BrandingPayload, CalendarPayload, CalendarProvider, EventPayload, MailProvider, MessageBody,
    MessagePayload, PluginError, ProviderBranding, ProviderPlugin, SendMessagePayload, SyncResult,
};

/// A reference/test implementation of a provider plugin.
/// Used by the unit tests and as a template for real WASM plugins.
pub struct MockProviderPlugin {
    id: String,
    branding: BrandingPayload,
}

impl MockProviderPlugin {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            branding: BrandingPayload {
                name: name.to_string(),
                button_text: format!("Continue with {}", name),
                button_color: "#4285F4".to_string(),
                icon_svg: "<svg></svg>".to_string(),
            },
        }
    }
}

impl ProviderBranding for MockProviderPlugin {
    fn get_branding(&self) -> BrandingPayload {
        self.branding.clone()
    }
}

#[async_trait]
impl MailProvider for MockProviderPlugin {
    async fn sync_mail(
        &self,
        _auth_token: &str,
        _cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError> {
        Ok(SyncResult {
            messages: vec![MessagePayload {
                id: "mock-1".to_string(),
                external_id: "mock-1".to_string(),
                thread_id: "mock-thread-1".to_string(),
                subject: Some("Mock message".to_string()),
                sender_name: Some("Mock Sender".to_string()),
                sender_email: "mock@example.com".to_string(),
                recipients: "user@example.com".to_string(),
                date_sent: 0,
                date_received: 0,
                snippet: Some("This is a mock message".to_string()),
                labels: None,
                is_read: false,
            }],
            next_cursor: String::new(),
        })
    }

    async fn fetch_message_body(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<MessageBody, PluginError> {
        Ok(MessageBody {
            body_text: Some("Mock body text".to_string()),
            body_html: Some("<p>Mock body html</p>".to_string()),
        })
    }

    async fn download_attachment(
        &self,
        _auth_token: &str,
        _external_message_id: &str,
        _external_attachment_id: &str,
    ) -> Result<Vec<u8>, PluginError> {
        Ok(b"mock attachment".to_vec())
    }

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

    async fn delete_message(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn send_message(
        &self,
        _auth_token: &str,
        _payload: SendMessagePayload,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn archive_message(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn update_message_labels(
        &self,
        _auth_token: &str,
        _external_id: &str,
        _labels: Vec<String>,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn mark_as_read(
        &self,
        _auth_token: &str,
        _external_id: &str,
        _is_read: bool,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl CalendarProvider for MockProviderPlugin {
    async fn fetch_calendars(
        &self,
        _auth_token: &str,
    ) -> Result<Vec<CalendarPayload>, PluginError> {
        Ok(vec![CalendarPayload {
            id: "mock-cal-1".to_string(),
            name: "Mock Calendar".to_string(),
            color: Some("#4285F4".to_string()),
            is_primary: true,
        }])
    }

    async fn fetch_events(
        &self,
        _auth_token: &str,
        _start_time: i64,
        _end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError> {
        Ok(vec![EventPayload {
            id: "mock-event-1".to_string(),
            external_id: "mock-event-1".to_string(),
            title: "Mock Event".to_string(),
            description: None,
            location: None,
            start_time: 0,
            end_time: 3600,
            is_all_day: false,
            recurrence_rules: None,
            organizer_email: None,
            organizer_name: None,
            attendees: None,
            status: Some("confirmed".to_string()),
        }])
    }

    async fn mutate_event(
        &self,
        _auth_token: &str,
        _action: &str,
        _payload: &EventPayload,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn delete_event(&self, _auth_token: &str, _external_id: &str) -> Result<(), PluginError> {
        Ok(())
    }
}

impl ProviderPlugin for MockProviderPlugin {
    fn id(&self) -> &str {
        &self.id
    }
}
