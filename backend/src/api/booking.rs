use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::auth::AuthUser;
use super::booking_slots::{SLOT_STEP_SECS, compute_free_slots};
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::{BookingPage, Calendar, CalendarEvent};
use crate::core::repository::{AccountRepository, BookingPageRepository, CalendarRepository};
use crate::core::types::DbUuid;
use crate::db::pool::DbPool;

const MAX_RANGE_SECS: i64 = 31 * 24 * 3600;

fn booking_repo(state: &AppState) -> Box<dyn BookingPageRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => Box::new(
            crate::db::sqlite::booking_repository::SqliteBookingRepository::new(pool.clone()),
        ),
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::booking_repository::PostgresBookingRepository::new(pool.clone()),
        ),
    }
}

fn event_repo(state: &AppState) -> Box<dyn crate::core::repository::EventRepository> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            Box::new(crate::db::sqlite::event_repository::SqliteEventRepository::new(pool.clone()))
        }
        DbPool::Postgres(pool) => Box::new(
            crate::db::postgres::event_repository::PostgresEventRepository::new(pool.clone()),
        ),
    }
}

fn new_slug() -> String {
    Uuid::new_v4().simple().to_string()
}

fn validate_page_params(
    name: &str,
    duration_mins: i32,
    buffer_mins: i32,
    window_days: i32,
) -> Result<String, KestrelError> {
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 80 {
        return Err(KestrelError::BadRequest(
            "Name must be 1-80 characters".to_string(),
        ));
    }
    if !(5..=720).contains(&duration_mins) {
        return Err(KestrelError::BadRequest(
            "duration_mins must be 5-720".to_string(),
        ));
    }
    if !(0..=120).contains(&buffer_mins) {
        return Err(KestrelError::BadRequest(
            "buffer_mins must be 0-120".to_string(),
        ));
    }
    if !(1..=60).contains(&window_days) {
        return Err(KestrelError::BadRequest(
            "window_days must be 1-60".to_string(),
        ));
    }
    Ok(name)
}

async fn owned_calendar(
    state: &AppState,
    user_id: Uuid,
    calendar_id: Uuid,
) -> Result<Calendar, KestrelError> {
    let cal = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::calendar_repository::SqliteCalendarRepository::new(pool.clone())
                .find_by_id(calendar_id)
                .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::calendar_repository::PostgresCalendarRepository::new(pool.clone())
                .find_by_id(calendar_id)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;
    let account = match &state.db {
        DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )
            .find_by_id(cal.account_id.0)
            .await?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.jwt_secret.clone(),
            )
            .find_by_id(cal.account_id.0)
            .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".to_string()))?;
    if account.user_id.0 != user_id {
        return Err(KestrelError::Forbidden(
            "Calendar does not belong to you".to_string(),
        ));
    }
    Ok(cal)
}

async fn owned_page(
    state: &AppState,
    user_id: Uuid,
    page_id: Uuid,
) -> Result<BookingPage, KestrelError> {
    let page = booking_repo(state)
        .find_by_id(page_id)
        .await
        .map_err(KestrelError::from)?;
    let page = page.ok_or_else(|| KestrelError::NotFound("Booking page not found".to_string()))?;
    if page.user_id.0 != user_id {
        return Err(KestrelError::Forbidden(
            "Booking page does not belong to you".to_string(),
        ));
    }
    Ok(page)
}

async fn public_page_or_404(state: &AppState, slug: &str) -> Result<BookingPage, KestrelError> {
    let page = booking_repo(state)
        .find_by_slug(slug)
        .await
        .map_err(KestrelError::from)?;
    match page {
        Some(p) if p.is_active => Ok(p),
        _ => Err(KestrelError::NotFound("Booking page not found".to_string())),
    }
}

// --- Owner DTOs ---

#[derive(Debug, Deserialize, specta::Type)]
pub struct CreateBookingPageRequest {
    pub calendar_id: Uuid,
    pub name: String,
    pub duration_mins: i32,
    pub buffer_mins: Option<i32>,
    pub window_days: Option<i32>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct UpdateBookingPageRequest {
    pub calendar_id: Option<Uuid>,
    pub name: Option<String>,
    pub duration_mins: Option<i32>,
    pub buffer_mins: Option<i32>,
    pub window_days: Option<i32>,
    pub is_active: Option<bool>,
}

pub async fn create_page(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<CreateBookingPageRequest>,
) -> Result<(StatusCode, Json<BookingPage>), KestrelError> {
    let cal = owned_calendar(&state, user_id, body.calendar_id).await?;
    let name = validate_page_params(
        &body.name,
        body.duration_mins,
        body.buffer_mins.unwrap_or(0),
        body.window_days.unwrap_or(14),
    )?;
    let now = chrono::Utc::now().timestamp();
    let page = BookingPage {
        id: DbUuid::new(Uuid::new_v4()),
        user_id: DbUuid::new(user_id),
        calendar_id: cal.id,
        name,
        slug: new_slug(),
        duration_mins: body.duration_mins,
        buffer_mins: body.buffer_mins.unwrap_or(0),
        window_days: body.window_days.unwrap_or(14),
        is_active: true,
        created_at: now,
        updated_at: now,
    };
    booking_repo(&state)
        .create(&page)
        .await
        .map_err(KestrelError::from)?;
    Ok((StatusCode::CREATED, Json(page)))
}

pub async fn list_pages(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<BookingPage>>, KestrelError> {
    let pages = booking_repo(&state)
        .list_by_user(user_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(pages))
}

pub async fn get_page(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(page_id): Path<Uuid>,
) -> Result<Json<BookingPage>, KestrelError> {
    Ok(Json(owned_page(&state, user_id, page_id).await?))
}

pub async fn update_page(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(page_id): Path<Uuid>,
    Json(body): Json<UpdateBookingPageRequest>,
) -> Result<Json<BookingPage>, KestrelError> {
    let mut page = owned_page(&state, user_id, page_id).await?;
    if let Some(cal_id) = body.calendar_id {
        let cal = owned_calendar(&state, user_id, cal_id).await?;
        page.calendar_id = cal.id;
    }
    let name = body.name.as_deref().unwrap_or(&page.name);
    page.name = validate_page_params(
        name,
        body.duration_mins.unwrap_or(page.duration_mins),
        body.buffer_mins.unwrap_or(page.buffer_mins),
        body.window_days.unwrap_or(page.window_days),
    )?;
    if let Some(d) = body.duration_mins {
        page.duration_mins = d;
    }
    if let Some(b) = body.buffer_mins {
        page.buffer_mins = b;
    }
    if let Some(w) = body.window_days {
        page.window_days = w;
    }
    if let Some(a) = body.is_active {
        page.is_active = a;
    }
    page.updated_at = chrono::Utc::now().timestamp();
    booking_repo(&state)
        .update(&page)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(page))
}

pub async fn rotate_slug(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(page_id): Path<Uuid>,
) -> Result<Json<BookingPage>, KestrelError> {
    let mut page = owned_page(&state, user_id, page_id).await?;
    page.slug = new_slug();
    page.updated_at = chrono::Utc::now().timestamp();
    booking_repo(&state)
        .update(&page)
        .await
        .map_err(KestrelError::from)?;
    Ok(Json(page))
}

pub async fn delete_page(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(page_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    owned_page(&state, user_id, page_id).await?;
    booking_repo(&state)
        .delete(page_id)
        .await
        .map_err(KestrelError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Public DTOs (no auth; slug is the credential) ---

#[derive(Debug, Serialize, specta::Type)]
pub struct PublicBookingDto {
    pub name: String,
    pub duration_mins: i32,
    pub buffer_mins: i32,
    pub window_days: i32,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct SlotDto {
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct SlotsResponse {
    pub slots: Vec<SlotDto>,
}

#[derive(Debug, Deserialize, specta::Type)]
pub struct BookSlotRequest {
    pub guest_name: String,
    pub guest_email: String,
    #[specta(type = f64)]
    pub start_time: i64,
}

#[derive(Debug, Serialize, specta::Type)]
pub struct BookSlotResponse {
    pub event_id: Uuid,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
}

pub async fn public_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicBookingDto>, KestrelError> {
    let page = public_page_or_404(&state, &slug).await?;
    Ok(Json(PublicBookingDto {
        name: page.name,
        duration_mins: page.duration_mins,
        buffer_mins: page.buffer_mins,
        window_days: page.window_days,
    }))
}

async fn busy_blocks(
    state: &AppState,
    page: &BookingPage,
    from: i64,
    to: i64,
) -> Result<Vec<(i64, i64)>, KestrelError> {
    let events = event_repo(state)
        .list_range(page.user_id.0, from, to, Some(page.calendar_id.0))
        .await
        .map_err(KestrelError::from)?;
    Ok(events
        .into_iter()
        .map(|e| (e.start_time, e.end_time))
        .collect())
}

pub async fn public_slots(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<SlotsResponse>, KestrelError> {
    let page = public_page_or_404(&state, &slug).await?;
    let now = chrono::Utc::now().timestamp();
    let mut from = params
        .get("from")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(now)
        .max(now);
    // Snap down to the advertised grid so listed slots and the
    // book-time alignment check always agree.
    from -= from % SLOT_STEP_SECS;
    let mut to = params
        .get("to")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(now + page.window_days as i64 * 86400);
    if to - from > MAX_RANGE_SECS {
        to = from + MAX_RANGE_SECS;
    }
    if to <= from {
        return Err(KestrelError::BadRequest("Empty time range".to_string()));
    }
    let busy = busy_blocks(&state, &page, from, to).await?;
    let duration = page.duration_mins as i64 * 60;
    let buffer = page.buffer_mins as i64 * 60;
    let cutoff = now + 60;
    let slots = compute_free_slots(from, to, &busy, duration, buffer)
        .into_iter()
        .filter(|(s, _)| *s >= cutoff)
        .map(|(start_time, end_time)| SlotDto {
            start_time,
            end_time,
        })
        .collect();
    Ok(Json(SlotsResponse { slots }))
}

fn valid_email(s: &str) -> bool {
    if s.len() < 3 || s.len() > 254 {
        return false;
    }
    if s.bytes().any(|b| b <= 32 || b == 127) {
        return false;
    }
    let mut parts = s.split('@');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(local), Some(domain), None) => {
            !local.is_empty()
                && domain.contains('.')
                && !domain.starts_with('.')
                && !domain.ends_with('.')
        }
        _ => false,
    }
}

/// Strip angle brackets so guest input stays inert wherever stored
/// event fields are rendered.
fn plain(s: &str) -> String {
    s.chars().filter(|c| *c != '<' && *c != '>').collect()
}

pub async fn book_slot(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(body): Json<BookSlotRequest>,
) -> Result<(StatusCode, Json<BookSlotResponse>), KestrelError> {
    let page = public_page_or_404(&state, &slug).await?;
    let guest_name = plain(body.guest_name.trim());
    if guest_name.is_empty() || guest_name.chars().count() > 80 {
        return Err(KestrelError::BadRequest(
            "guest_name must be 1-80 characters".to_string(),
        ));
    }
    let guest_email = plain(body.guest_email.trim()).to_lowercase();
    if !valid_email(&guest_email) {
        return Err(KestrelError::BadRequest("Invalid guest_email".to_string()));
    }
    let now = chrono::Utc::now().timestamp();
    let duration = page.duration_mins as i64 * 60;
    let buffer = page.buffer_mins as i64 * 60;
    let window_end = now + page.window_days as i64 * 86400;
    if body.start_time < now + 60 || body.start_time + duration > window_end {
        return Err(KestrelError::BadRequest(
            "start_time is outside the booking window".to_string(),
        ));
    }
    if body.start_time % SLOT_STEP_SECS != 0 {
        return Err(KestrelError::BadRequest(
            "start_time must align to the 15-minute grid".to_string(),
        ));
    }
    // No availability pre-check here on purpose: the single atomic
    // insert below is the only gate, so concurrent bookings cannot
    // slip through a check-then-insert gap.
    let event_id = Uuid::new_v4();
    let cal = match &state.db {
        DbPool::Sqlite(pool) => {
            { crate::db::sqlite::calendar_repository::SqliteCalendarRepository::new(pool.clone()) }
                .find_by_id(page.calendar_id.0)
                .await
                .map_err(KestrelError::from)?
        }
        DbPool::Postgres(pool) => {
            crate::db::postgres::calendar_repository::PostgresCalendarRepository::new(pool.clone())
        }
        .find_by_id(page.calendar_id.0)
        .await
        .map_err(KestrelError::from)?,
    }
    .ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;
    let event = CalendarEvent {
        id: DbUuid::new(event_id),
        account_id: cal.account_id,
        calendar_id: page.calendar_id,
        external_id: format!("booking-{event_id}"),
        title: page.name.clone(),
        description: Some(format!("Booked by {guest_name} <{guest_email}>")),
        location: None,
        start_time: body.start_time,
        end_time: body.start_time + duration,
        is_all_day: false,
        recurrence_rules: None,
        organizer_email: None,
        organizer_name: None,
        attendees: Some(
            serde_json::json!([{"email": guest_email, "name": guest_name}]).to_string(),
        ),
        status: Some("confirmed".to_string()),
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };
    let inserted = booking_repo(&state)
        .insert_event_if_free(
            &event,
            body.start_time - buffer,
            body.start_time + duration + buffer,
        )
        .await
        .map_err(KestrelError::from)?;
    if !inserted {
        return Err(KestrelError::Conflict("Slot was just taken".to_string()));
    }
    Ok((
        StatusCode::CREATED,
        Json(BookSlotResponse {
            event_id,
            start_time: event.start_time,
            end_time: event.end_time,
        }),
    ))
}

/// Minimal self-contained guest page so a token link works in any
/// browser with no account. Uses the public JSON endpoints above.
pub async fn guest_page(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, KestrelError> {
    let page = public_page_or_404(&state, &slug).await?;
    let html = format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Book: {name}</title></head><body>
<h1>Book: {name}</h1>
<p>{duration} minute meeting. Times shown in your local timezone.</p>
<label>Date <input type="date" id="d"></label>
<button id="load">Show times</button>
<ul id="slots"></ul>
<h2>Your details</h2>
<label>Name <input id="n"></label><br>
<label>Email <input id="e" type="email"></label><br>
<p id="msg"></p>
<script>
const slug = "{slug}";
const pad = n => String(n).padStart(2, '0');
document.getElementById('load').onclick = async () => {{
  const d = document.getElementById('d').value;
  if (!d) return;
  const from = Math.floor(new Date(d + 'T00:00:00').getTime() / 1000);
  const to = from + 86400;
  const r = await fetch(`/api/book/${{slug}}/slots?from=${{from}}&to=${{to}}`);
  const j = await r.json();
  const ul = document.getElementById('slots');
  ul.innerHTML = '';
  (j.slots || []).forEach(s => {{
    const li = document.createElement('li');
    const t = new Date(s.start_time * 1000);
    const b = document.createElement('button');
    b.textContent = pad(t.getHours()) + ':' + pad(t.getMinutes());
    b.onclick = async () => {{
      const name = document.getElementById('n').value.trim();
      const email = document.getElementById('e').value.trim();
      if (!name || !email) {{
        document.getElementById('msg').textContent = 'Enter name and email first.';
        return;
      }}
      const br = await fetch(`/api/book/${{slug}}`, {{
        method: 'POST',
        headers: {{'content-type': 'application/json'}},
        body: JSON.stringify({{guest_name: name, guest_email: email, start_time: s.start_time}})
      }});
      document.getElementById('msg').textContent =
        br.status === 201 ? 'Booked!' : 'That slot was just taken, pick another.';
    }};
    li.appendChild(b);
    ul.appendChild(li);
  }});
}};
</script></body></html>"#,
        name = html_escape(&page.name),
        duration = page.duration_mins,
        slug = html_escape(&slug),
    );
    Ok(Html(html))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_params_reject_garbage() {
        assert!(validate_page_params("", 30, 0, 14).is_err());
        assert!(validate_page_params("x", 3, 0, 14).is_err());
        assert!(validate_page_params("x", 30, 200, 14).is_err());
        assert!(validate_page_params("x", 30, 0, 99).is_err());
        assert!(validate_page_params("Coffee chat", 30, 10, 14).is_ok());
    }

    #[test]
    fn escape_neutralizes_markup() {
        assert_eq!(html_escape("<b>&\"'"), "&lt;b&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn email_check_rejects_malformed() {
        assert!(valid_email("ada@example.com"));
        assert!(!valid_email("nope"));
        assert!(!valid_email("a@b"));
        assert!(!valid_email("a @b.com"));
        assert!(!valid_email("@b.com"));
        assert!(!valid_email("a@.com"));
        assert!(!valid_email("a@@b.com"));
    }

    #[test]
    fn plain_strips_angle_brackets() {
        assert_eq!(plain("A <b>\"q\"</b>"), "A b\"q\"/b");
    }
}
