#![allow(unsafe_op_in_unsafe_fn)]
use wit_bindgen::generate;
use serde_json::{Value, json};
use chrono::DateTime;
use base64::{Engine as _, engine::general_purpose::{STANDARD as b64, URL_SAFE as b64url}};

generate!({
    world: "kestrel-plugin",
    path: "../../wit",
});

struct GmailPlugin;

impl exports::kestrel::provider::provider_branding::Guest for GmailPlugin {
    fn get_branding() -> exports::kestrel::provider::provider_branding::BrandingPayload {
        exports::kestrel::provider::provider_branding::BrandingPayload {
            name: "Gmail".to_string(),
            button_text: "Sign in with Google".to_string(),
            button_color: "#EA4335".to_string(),
            icon_svg: "<svg></svg>".to_string(),
        }
    }
}

use kestrel::provider::http_client::{HttpRequest, request};
use exports::kestrel::provider::mail_provider::{Guest as MailGuest, SyncResult, MessageBody, SendMessagePayload, MessagePayload};
use exports::kestrel::provider::calendar_provider::{Guest as CalendarGuest, CalendarPayload, EventPayload};

fn get_header<'a>(headers: &'a [Value], name: &str) -> Option<&'a str> {
    headers.iter().find(|h| {
        h["name"].as_str().map(|n| n.eq_ignore_ascii_case(name)).unwrap_or(false)
    }).and_then(|h| h["value"].as_str())
}

fn parse_gmail_message(v: &Value) -> Option<MessagePayload> {
    let id = v["id"].as_str()?.to_string();
    let thread_id = v["threadId"].as_str().unwrap_or(&id).to_string();
    let snippet = v["snippet"].as_str().map(|s| s.to_string());
    
    let internal_date = v["internalDate"].as_str().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    let date_received = internal_date / 1000;
    
    let payload = &v["payload"];
    let headers = payload["headers"].as_array()?;
    
    let subject = get_header(headers, "Subject").map(|s| s.to_string());
    
    let from = get_header(headers, "From").unwrap_or("");
    let (sender_name, sender_email) = if let Some(idx) = from.find('<') {
        let name = from[..idx].trim().trim_matches('"').to_string();
        let email = from[idx+1..from.len().saturating_sub(1)].to_string();
        (Some(name), email)
    } else {
        (None, from.to_string())
    };
    
    let to = get_header(headers, "To").unwrap_or("");
    
    let date_str = get_header(headers, "Date").unwrap_or("");
    let date_sent = DateTime::parse_from_rfc2822(date_str).map(|dt| dt.timestamp()).unwrap_or(date_received);
    
    let is_read = !v["labelIds"].as_array()
        .map(|arr| arr.iter().any(|l| l.as_str() == Some("UNREAD")))
        .unwrap_or(false);

    Some(MessagePayload {
        id: id.clone(),
        external_id: id,
        thread_id,
        subject,
        sender_name,
        sender_email,
        recipients: to.to_string(),
        date_sent,
        date_received,
        snippet,
        labels: None,
        is_read,
    })
}

fn fetch_full_message(auth_token: &str, msg_id: &str) -> Result<Value, String> {
    let req = HttpRequest {
        method: "GET".to_string(),
        url: format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}", msg_id),
        headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
        body: None,
    };
    let res = request(&req)?;
    if res.status != 200 {
        return Err(format!("Failed to fetch message: HTTP {}", res.status));
    }
    serde_json::from_slice(&res.body).map_err(|_| "Failed to parse JSON".into())
}

impl MailGuest for GmailPlugin {
    fn sync_mail(auth_token: String, cursor: Option<String>) -> Result<SyncResult, String> {
        let mut url = "https://gmail.googleapis.com/gmail/v1/users/me/messages?maxResults=20".to_string();
        if let Some(c) = cursor {
            url = format!("{}&pageToken={}", url, c);
        }

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
        if let Some(msg_list) = json["messages"].as_array() {
            for m in msg_list {
                if let Some(id) = m["id"].as_str() {
                    // Fetch full message
                    if let Ok(full_msg) = fetch_full_message(&auth_token, id) {
                        if let Some(parsed) = parse_gmail_message(&full_msg) {
                            messages.push(parsed);
                        }
                    }
                }
            }
        }
        
        let next_cursor = json["nextPageToken"].as_str().map(|s| s.to_string()).unwrap_or_default();
        
        Ok(SyncResult {
            messages,
            next_cursor,
        })
    }
    
    fn fetch_message_body(auth_token: String, external_id: String) -> Result<MessageBody, String> {
        let full_msg = fetch_full_message(&auth_token, &external_id)?;
        
        let mut body_text = None;
        let mut body_html = None;
        
        fn walk_parts(part: &Value, text: &mut Option<String>, html: &mut Option<String>) {
            let mime_type = part["mimeType"].as_str().unwrap_or("");
            if mime_type == "text/plain" {
                if let Some(data) = part["body"]["data"].as_str() {
                    if let Ok(decoded) = b64url.decode(data) {
                        *text = String::from_utf8(decoded).ok();
                    }
                }
            } else if mime_type == "text/html" {
                if let Some(data) = part["body"]["data"].as_str() {
                    if let Ok(decoded) = b64url.decode(data) {
                        *html = String::from_utf8(decoded).ok();
                    }
                }
            }
            if let Some(parts) = part["parts"].as_array() {
                for p in parts {
                    walk_parts(p, text, html);
                }
            }
        }
        
        walk_parts(&full_msg["payload"], &mut body_text, &mut body_html);
        
        Ok(MessageBody {
            body_text,
            body_html,
        })
    }
    
    fn download_attachment(auth_token: String, external_message_id: String, external_attachment_id: String) -> Result<Vec<u8>, String> {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/attachments/{}", external_message_id, external_attachment_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to download attachment: HTTP {}", res.status));
        }
        
        if let Ok(json) = serde_json::from_slice::<Value>(&res.body) {
            if let Some(data) = json["data"].as_str() {
                return b64url.decode(data).map_err(|_| "Failed to decode base64url".to_string());
            }
        }
        
        Err("Invalid attachment format".to_string())
    }

    fn delete_message(auth_token: String, external_id: String) -> Result<(), String> {
        let req = HttpRequest {
            method: "POST".to_string(),
            url: format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}/trash", external_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status == 200 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }

    fn send_message(
        auth_token: String,
        payload: SendMessagePayload,
    ) -> Result<(), String> {
        let boundary = "=_Kestrel_Boundary_123456789";
        let mut eml = String::new();
        
        // Headers
        eml.push_str(&format!("Subject: {}\r\n", payload.subject));
        eml.push_str(&format!("To: {}\r\n", payload.to.join(", ")));
        if let Some(cc) = payload.cc {
            eml.push_str(&format!("Cc: {}\r\n", cc.join(", ")));
        }
        if let Some(bcc) = payload.bcc {
            eml.push_str(&format!("Bcc: {}\r\n", bcc.join(", ")));
        }
        eml.push_str("MIME-Version: 1.0\r\n");
        eml.push_str(&format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n", boundary));
        eml.push_str("\r\n");
        
        // HTML Body Part
        eml.push_str(&format!("--{}\r\n", boundary));
        eml.push_str("Content-Type: text/html; charset=\"utf-8\"\r\n");
        eml.push_str("Content-Transfer-Encoding: base64\r\n");
        eml.push_str("\r\n");
        
        let b64_body = b64.encode(payload.body_html.as_bytes());
        // wrap at 76 chars
        for chunk in b64_body.as_bytes().chunks(76) {
            eml.push_str(std::str::from_utf8(chunk).unwrap());
            eml.push_str("\r\n");
        }
        eml.push_str("\r\n");
        
        // Attachments
        if let Some(atts) = payload.attachments {
            for a in atts {
                eml.push_str(&format!("--{}\r\n", boundary));
                eml.push_str(&format!("Content-Type: {}; name=\"{}\"\r\n", a.content_type, a.filename));
                eml.push_str(&format!("Content-Disposition: attachment; filename=\"{}\"\r\n", a.filename));
                eml.push_str("Content-Transfer-Encoding: base64\r\n");
                eml.push_str("\r\n");
                
                let b64_att = b64.encode(&a.content);
                for chunk in b64_att.as_bytes().chunks(76) {
                    eml.push_str(std::str::from_utf8(chunk).unwrap());
                    eml.push_str("\r\n");
                }
                eml.push_str("\r\n");
            }
        }
        
        // End Boundary
        eml.push_str(&format!("--{}--\r\n", boundary));
        
        let b64_email = b64url.encode(eml.as_bytes());
        let json_body = json!({ "raw": b64_email });
        let body_bytes = serde_json::to_vec(&json_body).unwrap();
        
        let req = HttpRequest {
            method: "POST".to_string(),
            url: "https://gmail.googleapis.com/gmail/v1/users/me/messages/send".to_string(),
            headers: vec![
                ("Authorization".to_string(), format!("Bearer {}", auth_token)),
                ("Content-Type".to_string(), "application/json".to_string())
            ],
            body: Some(body_bytes),
        };
        
        let res = request(&req)?;
        if res.status == 200 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }
}

// ── Calendar helpers (Gmail) ────────────────────────────────────

/// Fetch the primary calendar id for this account (falls back to "primary").
fn gmail_primary_calendar_id(auth_token: &str) -> Result<String, String> {
    let req = HttpRequest {
        method: "GET".to_string(),
        url: "https://www.googleapis.com/calendar/v3/users/me/calendarList?maxResults=100".to_string(),
        headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
        body: None,
    };
    let res = request(&req)?;
    if res.status != 200 {
        return Err(format!("Failed to list calendars: HTTP {}", res.status));
    }
    let json: Value = serde_json::from_slice(&res.body).map_err(|_| "Failed to parse calendar list")?;
    if let Some(items) = json["items"].as_array() {
        for item in items {
            if item["primary"].as_bool().unwrap_or(false) {
                if let Some(id) = item["id"].as_str() {
                    return Ok(id.to_string());
                }
            }
        }
        // Fall back to the first calendar id
        if let Some(id) = items.first().and_then(|i| i["id"].as_str()) {
            return Ok(id.to_string());
        }
    }
    Ok("primary".to_string())
}

/// Convert a Gmail calendar item into a CalendarPayload.
fn parse_gmail_calendar(item: &Value) -> Option<CalendarPayload> {
    let id = item["id"].as_str()?.to_string();
    let name = item["summary"].as_str().map(|s| s.to_string()).unwrap_or_else(|| id.clone());
    let color = item["backgroundColor"].as_str().map(|s| s.to_string());
    let is_primary = item["primary"].as_bool().unwrap_or(false);
    Some(CalendarPayload {
        id,
        name,
        color,
        is_primary,
    })
}

/// Convert a Gmail event item into an EventPayload.
fn parse_gmail_event(item: &Value) -> Option<EventPayload> {
    let id = item["id"].as_str()?.to_string();
    let external_id = id.clone();
    let title = item["summary"].as_str().map(|s| s.to_string()).unwrap_or_else(|| "(no title)".to_string());
    let description = item["description"].as_str().map(|s| s.to_string());
    let location = item["location"].as_str().map(|s| s.to_string());

    // Gmail all-day events use `date`, timed events use `dateTime`.
    let start = &item["start"];
    let end = &item["end"];
    let start_date = start["date"].as_str();
    let end_date = end["date"].as_str();
    let start_dt = start["dateTime"].as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.timestamp()));
    let end_dt = end["dateTime"].as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.timestamp()));

    let (start_time, end_time, is_all_day) = if start_dt.is_some() && end_dt.is_some() {
        (start_dt.unwrap(), end_dt.unwrap(), false)
    } else if let (Some(sd), Some(ed)) = (start_date, end_date) {
        // All-day: Gmail end date is exclusive; subtract one day.
        let s = chrono::NaiveDate::parse_from_str(sd, "%Y-%m-%d").ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .and_then(|dt| dt.and_utc().timestamp().into());
        let e = chrono::NaiveDate::parse_from_str(ed, "%Y-%m-%d").ok()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .and_then(|dt| dt.and_utc().timestamp().into());
        (s.unwrap_or(0), e.unwrap_or(0), true)
    } else {
        return None;
    };

    // Recurrence rules (Gmail returns a list of RRULE strings)
    let recurrence_rules = item["recurrence"].as_array()
        .and_then(|arr| arr.first())
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    // Organizer
    let organizer_email = item["organizer"]["email"].as_str().map(|s| s.to_string());
    let organizer_name = item["organizer"]["displayName"].as_str().map(|s| s.to_string());

    // Attendees as a JSON array string
    let attendees = item["attendees"].as_array().map(|arr| {
        let mapped: Vec<Value> = arr.iter().map(|a| {
            json!({
                "email": a["email"].as_str().unwrap_or(""),
                "name": a["displayName"].as_str(),
                "responseStatus": a["responseStatus"].as_str(),
            })
        }).collect();
        serde_json::to_string(&mapped).unwrap_or_else(|_| "[]".to_string())
    });

    let status = item["status"].as_str().map(|s| s.to_string());

    Some(EventPayload {
        id,
        external_id,
        title,
        description,
        location,
        start_time,
        end_time,
        is_all_day,
        recurrence_rules,
        organizer_email,
        organizer_name,
        attendees,
        status,
    })
}

impl CalendarGuest for GmailPlugin {
    fn fetch_calendars(auth_token: String) -> Result<Vec<CalendarPayload>, String> {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: "https://www.googleapis.com/calendar/v3/users/me/calendarList?maxResults=100".to_string(),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to list calendars: HTTP {}", res.status));
        }
        let json: Value = serde_json::from_slice(&res.body).map_err(|_| "Failed to parse calendar list")?;
        let mut calendars = Vec::new();
        if let Some(items) = json["items"].as_array() {
            for item in items {
                if let Some(c) = parse_gmail_calendar(item) {
                    calendars.push(c);
                }
            }
        }
        Ok(calendars)
    }

    fn fetch_events(auth_token: String, start_time: i64, end_time: i64) -> Result<Vec<EventPayload>, String> {
        let calendar_id = gmail_primary_calendar_id(&auth_token)?;
        let start = chrono::DateTime::from_timestamp(start_time, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        let end = chrono::DateTime::from_timestamp(end_time, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        // Minimal percent-encoding for query params (RFC3339 contains ':' and '+')
        fn pct_encode(s: &str) -> String {
            let mut out = String::new();
            for b in s.bytes() {
                if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
                    out.push(b as char);
                } else {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
            out
        }
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime&maxResults=250",
            pct_encode(&calendar_id),
            pct_encode(&start),
            pct_encode(&end)
        );
        let req = HttpRequest {
            method: "GET".to_string(),
            url,
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status != 200 {
            return Err(format!("Failed to list events: HTTP {}", res.status));
        }
        let json: Value = serde_json::from_slice(&res.body).map_err(|_| "Failed to parse events")?;
        let mut events = Vec::new();
        if let Some(items) = json["items"].as_array() {
            for item in items {
                if let Some(e) = parse_gmail_event(item) {
                    events.push(e);
                }
            }
        }
        Ok(events)
    }
    
    fn mutate_event(auth_token: String, action: String, payload: EventPayload) -> Result<(), String> {
        use chrono::TimeZone;
        let start_dt = chrono::Utc.timestamp_opt(payload.start_time, 0).unwrap();
        let end_dt = chrono::Utc.timestamp_opt(payload.end_time, 0).unwrap();
        
        let mut event_json = json!({
            "summary": payload.title,
            "description": payload.description.unwrap_or_default(),
            "location": payload.location.unwrap_or_default(),
        });
        
        if payload.is_all_day {
            event_json["start"] = json!({ "date": start_dt.format("%Y-%m-%d").to_string() });
            event_json["end"] = json!({ "date": end_dt.format("%Y-%m-%d").to_string() });
        } else {
            event_json["start"] = json!({ "dateTime": start_dt.to_rfc3339() });
            event_json["end"] = json!({ "dateTime": end_dt.to_rfc3339() });
        }
        
        let (method, url) = if action == "create" {
            ("POST".to_string(), "https://www.googleapis.com/calendar/v3/calendars/primary/events".to_string())
        } else {
            ("PATCH".to_string(), format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", payload.external_id))
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
            url: format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", external_id),
            headers: vec![("Authorization".to_string(), format!("Bearer {}", auth_token))],
            body: None,
        };
        let res = request(&req)?;
        if res.status == 204 || res.status == 200 || res.status == 404 { Ok(()) } else { Err(format!("HTTP {}", res.status)) }
    }
}

export!(GmailPlugin);

impl exports::kestrel::provider::webhook_handler::Guest for GmailPlugin {
    fn handle_webhook(
        webhook_secret: String,
        query_params: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<exports::kestrel::provider::webhook_handler::WebhookResult, String> {
        // Verify the authentication token
        let token = query_params.iter().find(|(k, _)| k == "token").map(|(_, v)| v.as_str());
        if token != Some(webhook_secret.as_str()) {
            return Err("Unauthorized".to_string());
        }

        #[derive(serde::Deserialize)]
        struct GooglePubSubMessage { data: String }
        #[derive(serde::Deserialize)]
        struct GoogleWebhookPayload { message: GooglePubSubMessage }
        #[derive(serde::Deserialize)]
        struct GoogleWebhookData { #[serde(rename = "emailAddress")] email_address: String }

        let payload: GoogleWebhookPayload = serde_json::from_slice(&body).map_err(|_| "Invalid JSON".to_string())?;
        let decoded_data = b64.decode(payload.message.data).map_err(|_| "Invalid base64 payload".to_string())?;
        let data: GoogleWebhookData = serde_json::from_slice(&decoded_data).map_err(|_| "Invalid JSON payload".to_string())?;

        Ok(exports::kestrel::provider::webhook_handler::WebhookResult {
            status: 200,
            headers: vec![],
            body: b"OK".to_vec(),
            account_identifier: Some(data.email_address),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exports::kestrel::provider::webhook_handler::Guest;

    #[test]
    fn test_gmail_webhook_handler_success() {
        let secret = "my_secret".to_string();
        let query = vec![("token".to_string(), "my_secret".to_string())];
        let payload = r#"{
            "message": {
                "data": "ewogICJlbWFpbEFkZHJlc3MiOiAidGVzdEBnbWFpbC5jb20iLAogICJoaXN0b3J5SWQiOiAxMjM0NQp9"
            }
        }"#;

        let result = GmailPlugin::handle_webhook(secret, query, payload.as_bytes().to_vec()).unwrap();
        assert_eq!(result.status, 200);
        assert_eq!(result.account_identifier.unwrap(), "test@gmail.com");
    }

    #[test]
    fn test_gmail_webhook_handler_invalid_token() {
        let secret = "my_secret".to_string();
        let query = vec![("token".to_string(), "wrong".to_string())];
        let result = GmailPlugin::handle_webhook(secret, query, vec![]);
        assert!(result.is_err());
    }
}
