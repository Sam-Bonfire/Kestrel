use async_trait::async_trait;
use std::sync::Arc;
use wasmtime::component::Component;

use super::bindings::KestrelPlugin;
use super::traits::{
    BrandingPayload, CalendarPayload, CalendarProvider, EventPayload, MailProvider, MessageBody,
    MessagePayload, PluginError, ProviderBranding, ProviderPlugin, SendMessagePayload, SyncResult,
};
use super::wasm_runtime::{WasmEngine, WasmState};

pub struct WasmPlugin {
    id: String,
    branding: BrandingPayload,
    engine: Arc<WasmEngine>,
    component: Component,
}

impl WasmPlugin {
    pub async fn new(
        id: String,
        engine: Arc<WasmEngine>,
        component: Component,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Instantiate the plugin once to get the static branding info
        let mut store = engine.create_store();

        // Setup imports if needed
        // For now, we mock the client credentials import
        let (instance, _) =
            KestrelPlugin::instantiate_async(&mut store, &component, &engine.linker).await?;

        // Extract branding
        let branding_res = instance
            .kestrel_provider_provider_branding()
            .call_get_branding(&mut store)
            .await?;

        let branding = BrandingPayload {
            name: branding_res.name,
            button_text: branding_res.button_text,
            button_color: branding_res.button_color,
            icon_svg: branding_res.icon_svg,
        };

        Ok(Self {
            id,
            branding,
            engine,
            component,
        })
    }

    /// Helper to instantiate a fresh store and plugin instance for a stateless request.
    async fn instantiate(
        &self,
    ) -> Result<(wasmtime::Store<WasmState>, KestrelPlugin), PluginError> {
        let mut store = self.engine.create_store();
        let (instance, _) =
            KestrelPlugin::instantiate_async(&mut store, &self.component, &self.engine.linker)
                .await
                .map_err(|e| PluginError(e.to_string()))?;

        Ok((store, instance))
    }
}

impl ProviderBranding for WasmPlugin {
    fn get_branding(&self) -> BrandingPayload {
        self.branding.clone()
    }
}

#[async_trait]
impl MailProvider for WasmPlugin {
    async fn sync_mail(
        &self,
        auth_token: &str,
        cursor: Option<&str>,
    ) -> Result<SyncResult, PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_mail_provider()
            .call_sync_mail(&mut store, auth_token, cursor)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(res) => Ok(SyncResult {
                messages: res
                    .messages
                    .into_iter()
                    .map(|m| MessagePayload {
                        id: m.id,
                        external_id: m.external_id,
                        thread_id: m.thread_id,
                        subject: m.subject,
                        sender_name: m.sender_name,
                        sender_email: m.sender_email,
                        recipients: m.recipients,
                        date_sent: m.date_sent,
                        date_received: m.date_received,
                        snippet: m.snippet,
                        labels: m.labels,
                        is_read: m.is_read,
                    })
                    .collect(),
                next_cursor: res.next_cursor,
            }),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn fetch_message_body(
        &self,
        auth_token: &str,
        external_id: &str,
    ) -> Result<MessageBody, PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_mail_provider()
            .call_fetch_message_body(&mut store, auth_token, external_id)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(res) => Ok(MessageBody {
                body_text: res.body_text,
                body_html: res.body_html,
            }),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn delete_message(&self, auth_token: &str, external_id: &str) -> Result<(), PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_mail_provider()
            .call_delete_message(&mut store, auth_token, external_id)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn send_message(
        &self,
        auth_token: &str,
        payload: SendMessagePayload,
    ) -> Result<(), PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let wit_attachments = payload.attachments.map(|atts| {
            atts.into_iter().map(|a| {
                crate::plugins::bindings::exports::kestrel::provider::mail_provider::AttachmentPayload {
                    filename: a.filename,
                    content_type: a.content_type,
                    content: a.content,
                }
            }).collect::<Vec<_>>()
        });

        let wit_payload = crate::plugins::bindings::exports::kestrel::provider::mail_provider::SendMessagePayload {
            to: payload.to,
            cc: payload.cc,
            bcc: payload.bcc,
            subject: payload.subject,
            body_html: payload.body_html,
            attachments: wit_attachments,
        };

        let result = instance
            .kestrel_provider_mail_provider()
            .call_send_message(&mut store, auth_token, &wit_payload)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn archive_message(
        &self,
        _auth_token: &str,
        _external_id: &str,
    ) -> Result<(), PluginError> {
        Err(PluginError("Not implemented for WASM yet".to_string()))
    }

    async fn download_attachment(
        &self,
        auth_token: &str,
        external_message_id: &str,
        external_attachment_id: &str,
    ) -> Result<Vec<u8>, PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_mail_provider()
            .call_download_attachment(
                &mut store,
                auth_token,
                external_message_id,
                external_attachment_id,
            )
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(res) => Ok(res),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn update_message_labels(
        &self,
        _auth_token: &str,
        _external_id: &str,
        _labels: Vec<String>,
    ) -> Result<(), PluginError> {
        Err(PluginError("Not implemented for WASM yet".to_string()))
    }

    async fn mark_as_read(
        &self,
        _auth_token: &str,
        _external_id: &str,
        _is_read: bool,
    ) -> Result<(), PluginError> {
        Err(PluginError("Not implemented for WASM yet".to_string()))
    }
}

#[async_trait]
impl CalendarProvider for WasmPlugin {
    async fn fetch_calendars(&self, auth_token: &str) -> Result<Vec<CalendarPayload>, PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_calendar_provider()
            .call_fetch_calendars(&mut store, auth_token)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(res) => Ok(res
                .into_iter()
                .map(|c| CalendarPayload {
                    id: c.id,
                    name: c.name,
                    color: c.color,
                    is_primary: c.is_primary,
                })
                .collect()),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn fetch_events(
        &self,
        auth_token: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<EventPayload>, PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_calendar_provider()
            .call_fetch_events(&mut store, auth_token, start_time, end_time)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(res) => Ok(res
                .into_iter()
                .map(|e| EventPayload {
                    id: e.id,
                    external_id: e.external_id,
                    title: e.title,
                    description: e.description,
                    location: e.location,
                    start_time: e.start_time,
                    end_time: e.end_time,
                    is_all_day: e.is_all_day,
                    recurrence_rules: e.recurrence_rules,
                    organizer_email: e.organizer_email,
                    organizer_name: e.organizer_name,
                    attendees: e.attendees,
                    status: e.status,
                })
                .collect()),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn mutate_event(
        &self,
        auth_token: &str,
        action: &str,
        payload: &EventPayload,
    ) -> Result<(), PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        // Convert to WIT struct
        let wit_payload =
            super::bindings::exports::kestrel::provider::calendar_provider::EventPayload {
                id: payload.id.clone(),
                external_id: payload.external_id.clone(),
                title: payload.title.clone(),
                description: payload.description.clone(),
                location: payload.location.clone(),
                start_time: payload.start_time,
                end_time: payload.end_time,
                is_all_day: payload.is_all_day,
                recurrence_rules: payload.recurrence_rules.clone(),
                organizer_email: payload.organizer_email.clone(),
                organizer_name: payload.organizer_name.clone(),
                attendees: payload.attendees.clone(),
                status: payload.status.clone(),
            };

        let result = instance
            .kestrel_provider_calendar_provider()
            .call_mutate_event(&mut store, auth_token, action, &wit_payload)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError(e)),
        }
    }

    async fn delete_event(&self, auth_token: &str, external_id: &str) -> Result<(), PluginError> {
        let (mut store, instance) = self.instantiate().await?;

        let result = instance
            .kestrel_provider_calendar_provider()
            .call_delete_event(&mut store, auth_token, external_id)
            .await
            .map_err(|e| PluginError(e.to_string()))?;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(PluginError(e)),
        }
    }
}

impl ProviderPlugin for WasmPlugin {
    fn id(&self) -> &str {
        &self.id
    }
}
