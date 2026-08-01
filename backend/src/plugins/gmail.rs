use async_trait::async_trait;
use crate::plugins::traits::{
    BrandingPayload, CalendarPayload, CalendarProvider, EventPayload, MailProvider,
    MessageBody, PluginError, ProviderBranding, ProviderPlugin, SyncResult,
};

pub struct GmailProviderPlugin {
    pub client_id: String,
    pub client_secret: String,
}

impl GmailProviderPlugin {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }
}

#[async_trait]
impl ProviderBranding for GmailProviderPlugin {
    fn get_branding(&self) -> BrandingPayload {
        BrandingPayload {
            name: "Google".to_string(),
            button_text: "Sign in with Google".to_string(),
            button_color: "#EA4335".to_string(),
            icon_svg: "<svg>...</svg>".to_string(),
        }
    }
}

#[async_trait]
impl ProviderPlugin for GmailProviderPlugin {
    fn id(&self) -> &str {
        "gmail"
    }
}

#[async_trait]
impl MailProvider for GmailProviderPlugin {
    async fn sync_mail(
        &self,
        _auth_token: &str,
        _cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError> {
        Ok(SyncResult {
            messages: vec![],
            next_cursor: "gmail_cursor_next".to_string(),
        })
    }

    async fn fetch_message_body(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<MessageBody, PluginError> {
        Ok(MessageBody {
            body_text: Some("Real Gmail body text".to_string()),
            body_html: Some("<p>Real Gmail HTML</p>".to_string()),
        })
    }

    async fn delete_message(
        &self,
        _auth_token: &str,
        _external_id: &str,
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
impl CalendarProvider for GmailProviderPlugin {
    async fn fetch_calendars(
        &self,
        _auth_token: &str,
    ) -> Result<Vec<CalendarPayload>, PluginError> {
        Ok(vec![])
    }

    async fn fetch_events(
        &self,
        _auth_token: &str,
        _start_time: i64,
        _end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError> {
        Ok(vec![])
    }

    async fn mutate_event(
        &self,
        _auth_token: &str,
        _action: &str,
        payload: &EventPayload,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn delete_event(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}
