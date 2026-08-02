use wit_bindgen::generate;
use serde_json::{Value, json};
use chrono::{DateTime, Utc};
use base64::{Engine as _, engine::general_purpose::STANDARD as b64};

generate!({
    world: "kestrel-plugin",
    path: "../../wit",
});

struct OutlookPlugin;

impl exports::kestrel::provider::provider_branding::Guest for OutlookPlugin {
    fn get_branding() -> exports::kestrel::provider::provider_branding::BrandingPayload {
        exports::kestrel::provider::provider_branding::BrandingPayload {
            name: "Outlook".to_string(),
            button_text: "Sign in with Microsoft".to_string(),
            button_color: "#0078D4".to_string(),
            icon_svg: "<svg></svg>".to_string(),
        }
    }
}

use kestrel::provider::http_client::{HttpRequest, request};
use exports::kestrel::provider::mail_provider::{Guest as MailGuest, SyncResult, MessageBody, SendMessagePayload};
use exports::kestrel::provider::mail_provider::MessagePayload;
use exports::kestrel::provider::calendar_provider::{Guest as CalendarGuest, CalendarPayload, EventPayload};

fn parse_outlook_date(date_str: Option<&str>) -> i64 {
    if let Some(s) = date_str {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return dt.timestamp();
        }
    }
    0
}

fn map_message(v: &Value) -> Option<MessagePayload> {
    let id = v["id"].as_str()?.to_string();
    let thread_id = v["conversationId"].as_str().unwrap_or(&id).to_string();
    let subject = v["subject"].as_str().map(|s| s.to_string());
    
    let sender_email = v["sender"]["emailAddress"]["address"].as_str().unwrap_or("").to_string();
    let sender_name = v["sender"]["emailAddress"]["name"].as_str().map(|s| s.to_string());
    
    let mut recipients_list = Vec::new();
    if let Some(to) = v["toRecipients"].as_array() {
        for r in to {
            if let Some(addr) = r["emailAddress"]["address"].as_str() {
                recipients_list.push(addr.to_string());
            }
        }
    }
    let recipients = recipients_list.join(", ");
    
    let date_received = parse_outlook_date(v["receivedDateTime"].as_str());
    let date_sent = parse_outlook_date(v["sentDateTime"].as_str());
    
    let snippet = v["bodyPreview"].as_str().map(|s| s.to_string());
    let is_read = v["isRead"].as_bool().unwrap_or(false);

    Some(MessagePayload {
        id: id.clone(),
        external_id: id,
        thread_id,
        subject,
        sender_name,
        sender_email,
        recipients,
        date_sent,
        date_received,
        snippet,
        labels: None, // Graph API uses folders/categories, simplified for now
        is_read,
    })
}

impl MailGuest for OutlookPlugin {
    fn sync_mail(auth_token: String, cursor: Option<String>) -> Result<SyncResult, String> {
        let url = if let Some(c) = cursor {
            c
        } else {
            "https://graph.microsoft.com/v1.0/me/messages?$top=50&$select=id,conversationId,subject,sender,toRecipients,receivedDateTime,sentDateTime,bodyPreview,isRead".to_string()
        };

        let req = HttpRequest {
            method: "GET".to_string(),
            url,
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to sync mail: HTTP {}", res.status));
        }
        
        let json: Value = serde_json::from_slice(&res.body).map_err(|_| "Failed to parse JSON")?;
        
        let mut messages = Vec::new();
        if let Some(vals) = json["value"].as_array() {
            for v in vals {
                if let Some(m) = map_message(v) {
                    messages.push(m);
                }
            }
        }
        
        let next_cursor = json["@odata.nextLink"].as_str().map(|s| s.to_string()).unwrap_or_default();
        
        Ok(SyncResult {
            messages,
            next_cursor,
        })
    }
    
    fn fetch_message_body(auth_token: String, external_id: String) -> Result<MessageBody, String> {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("https://graph.microsoft.com/v1.0/me/messages/{}?$select=body", external_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to fetch message body: HTTP {}", res.status));
        }
        
        let json: Value = serde_json::from_slice(&res.body).map_err(|_| "Failed to parse JSON")?;
        let content_type = json["body"]["contentType"].as_str().unwrap_or("text");
        let content = json["body"]["content"].as_str().unwrap_or("").to_string();
        
        let (body_text, body_html) = if content_type.eq_ignore_ascii_case("html") {
            (None, Some(content))
        } else {
            (Some(content), None)
        };
        
        Ok(MessageBody {
            body_text,
            body_html,
        })
    }
    
    fn download_attachment(auth_token: String, external_message_id: String, external_attachment_id: String) -> Result<Vec<u8>, String> {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("https://graph.microsoft.com/v1.0/me/messages/{}/attachments/{}/$value", external_message_id, external_attachment_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to download attachment: HTTP {}", res.status));
        }
        
        Ok(res.body)
    }

    fn delete_message(auth_token: String, external_id: String) -> Result<(), String> {
        let req = HttpRequest {
            method: "DELETE".to_string(),
            url: format!("https://graph.microsoft.com/v1.0/me/messages/{}", external_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status == 204 || res.status == 200 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }

    fn send_message(
        auth_token: String,
        payload: SendMessagePayload,
    ) -> Result<(), String> {
        let mut to_recipients = Vec::new();
        for addr in payload.to {
            to_recipients.push(json!({ "emailAddress": { "address": addr } }));
        }

        let mut cc_recipients = Vec::new();
        if let Some(cc) = payload.cc {
            for addr in cc {
                cc_recipients.push(json!({ "emailAddress": { "address": addr } }));
            }
        }

        let mut bcc_recipients = Vec::new();
        if let Some(bcc) = payload.bcc {
            for addr in bcc {
                bcc_recipients.push(json!({ "emailAddress": { "address": addr } }));
            }
        }

        let mut attachments_json = Vec::new();
        if let Some(atts) = payload.attachments {
            for a in atts {
                attachments_json.push(json!({
                    "@odata.type": "#microsoft.graph.fileAttachment",
                    "name": a.filename,
                    "contentType": a.content_type,
                    "contentBytes": b64.encode(&a.content)
                }));
            }
        }

        let message = json!({
            "message": {
                "subject": payload.subject,
                "body": {
                    "contentType": "HTML",
                    "content": payload.body_html
                },
                "toRecipients": to_recipients,
                "ccRecipients": cc_recipients,
                "bccRecipients": bcc_recipients,
                "attachments": attachments_json
            }
        });

        let body_bytes = serde_json::to_vec(&message).map_err(|_| "Failed to serialize payload")?;

        let req = HttpRequest {
            method: "POST".to_string(),
            url: "https://graph.microsoft.com/v1.0/me/sendMail".to_string(),
            headers: vec![
                ("Authorization".to_string(), format!("Bearer {}", auth_token)),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(body_bytes),
        };
        
        let res = request(&req)?;
        if res.status == 202 || res.status == 200 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }
}

impl CalendarGuest for OutlookPlugin {
    fn fetch_calendars(_auth_token: String) -> Result<Vec<CalendarPayload>, String> { Ok(vec![]) }
    fn fetch_events(_auth_token: String, _start_time: i64, _end_time: i64) -> Result<Vec<EventPayload>, String> { Ok(vec![]) }
    
    fn mutate_event(auth_token: String, action: String, payload: EventPayload) -> Result<(), String> {
        // Use chrono's TimeZone to create UTC DateTime, then format
        use chrono::TimeZone;
        let start_dt = chrono::Utc.timestamp_opt(payload.start_time, 0).unwrap();
        let end_dt = chrono::Utc.timestamp_opt(payload.end_time, 0).unwrap();
        
        // Graph API requires the format: YYYY-MM-DDTHH:MM:SS
        let start_str = start_dt.format("%Y-%m-%dT%H:%M:%S").to_string();
        let end_str = end_dt.format("%Y-%m-%dT%H:%M:%S").to_string();

        let event_json = json!({
            "subject": payload.title,
            "body": {
                "contentType": "HTML",
                "content": payload.description.unwrap_or_default()
            },
            "location": {
                "displayName": payload.location.unwrap_or_default()
            },
            "start": {
                "dateTime": start_str,
                "timeZone": "UTC"
            },
            "end": {
                "dateTime": end_str,
                "timeZone": "UTC"
            },
            "isAllDay": payload.is_all_day
        });
        
        let (method, url) = if action == "create" {
            ("POST".to_string(), "https://graph.microsoft.com/v1.0/me/events".to_string())
        } else {
            ("PATCH".to_string(), format!("https://graph.microsoft.com/v1.0/me/events/{}", payload.external_id))
        };
        
        let body_bytes = serde_json::to_vec(&event_json).unwrap();
        
        let req = HttpRequest {
            method,
            url,
            headers: vec![
                ("Authorization".to_string(), format!("Bearer {}", auth_token)),
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(body_bytes),
        };
        
        let res = request(&req)?;
        if res.status >= 200 && res.status < 300 { Ok(()) } else { Err(format!("HTTP {} - {}", res.status, String::from_utf8_lossy(&res.body))) }
    }
    
    fn delete_event(auth_token: String, external_id: String) -> Result<(), String> {
        let req = HttpRequest {
            method: "DELETE".to_string(),
            url: format!("https://graph.microsoft.com/v1.0/me/events/{}", external_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status == 204 || res.status == 200 || res.status == 404 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }
}

export!(OutlookPlugin);
