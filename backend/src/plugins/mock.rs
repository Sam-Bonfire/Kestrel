use super::traits::{
    BrandingPayload, CalendarPayload, CalendarProvider, EventPayload, MailProvider, MessageBody,
    PluginError, ProviderBranding, ProviderPlugin, SyncResult,
};

/// A mock provider plugin for testing. Implements all plugin traits
/// with stub implementations that return predictable data.
pub struct MockProviderPlugin {
    id: String,
    name: String,
}

impl MockProviderPlugin {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

impl ProviderPlugin for MockProviderPlugin {
    fn id(&self) -> &str {
        &self.id
    }
}

impl ProviderBranding for MockProviderPlugin {
    fn get_branding(&self) -> BrandingPayload {
        BrandingPayload {
            name: self.name.clone(),
            button_text: format!("Continue with {}", self.name),
            button_color: "#4285F4".to_string(),
            icon_svg: "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\"><circle cx=\"12\" cy=\"12\" r=\"10\" fill=\"currentColor\"/></svg>".to_string(),
        }
    }
}

impl MailProvider for MockProviderPlugin {
    fn sync_mail(
        &self,
        _auth_token: &str,
        _cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError> {
        Ok(SyncResult {
            messages: vec![],
            next_cursor: String::new(),
        })
    }

    fn fetch_message_body(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<MessageBody, PluginError> {
        Ok(MessageBody {
            body_text: Some("Mock message body text".to_string()),
            body_html: Some("<p>Mock message body HTML</p>".to_string()),
        })
    }

    fn delete_message(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

impl CalendarProvider for MockProviderPlugin {
    fn fetch_calendars(
        &self,
        _auth_token: &str,
    ) -> Result<Vec<CalendarPayload>, PluginError> {
        Ok(vec![])
    }

    fn fetch_events(
        &self,
        _auth_token: &str,
        _start_time: i64,
        _end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError> {
        Ok(vec![])
    }

    fn mutate_event(
        &self,
        _auth_token: &str,
        _action: &str,
        _payload: &EventPayload,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn delete_event(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_branding() {
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        let branding = mock.get_branding();
        assert_eq!(branding.name, "Gmail");
        assert_eq!(branding.button_text, "Continue with Gmail");
    }

    #[test]
    fn test_mock_sync_mail() {
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        let result = mock.sync_mail("token", None).unwrap();
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_mock_fetch_body() {
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        let body = mock.fetch_message_body("token", "ext-123").unwrap();
        assert!(body.body_text.is_some());
        assert!(body.body_html.is_some());
    }

    #[test]
    fn test_mock_calendar() {
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        let calendars = mock.fetch_calendars("token").unwrap();
        assert!(calendars.is_empty());

        let events = mock.fetch_events("token", 0, 1000).unwrap();
        assert!(events.is_empty());
    }

    #[test]
    fn test_mock_delete() {
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        assert!(mock.delete_message("token", "ext-123").is_ok());
        assert!(mock.delete_event("token", "ext-123").is_ok());
    }
}
