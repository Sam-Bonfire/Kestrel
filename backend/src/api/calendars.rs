use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_TYPE, ETAG, HeaderMap, IF_NONE_MATCH, LAST_MODIFIED};
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{TimeZone, Utc};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::auth::AuthUser;
use super::router::AppState;
use crate::core::error::KestrelError;
use crate::core::models::{Calendar, CalendarEvent};
use crate::core::repository::{AccountRepository, CalendarRepository, EventRepository};
use crate::core::types::DbUuid;
use crate::db::pool::DbPool;
use crate::db::postgres::calendar_repository::PostgresCalendarRepository;
use crate::db::postgres::event_repository::PostgresEventRepository;
use crate::db::sqlite::calendar_repository::SqliteCalendarRepository;
use crate::db::sqlite::event_repository::SqliteEventRepository;

// --- K-048: GET /api/v1/calendars ---

#[derive(Serialize, specta::Type)]
pub struct CalendarSummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    #[specta(type = f64)]
    pub created_at: i64,
    #[specta(type = f64)]
    pub updated_at: i64,
}

#[derive(Serialize, specta::Type)]
pub struct CalendarListResponse {
    pub calendars: Vec<CalendarSummary>,
    #[specta(type = f64)]
    pub total: usize,
}

pub async fn list_calendars(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<CalendarListResponse>, KestrelError> {
    let calendars = list_calendars_from_db(&state, user_id).await?;
    let total = calendars.len();

    let summaries: Vec<CalendarSummary> = calendars
        .into_iter()
        .map(|c| CalendarSummary {
            id: c.id.0,
            account_id: c.account_id.0,
            external_id: c.external_id,
            name: c.name,
            color: c.color,
            is_primary: c.is_primary,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();

    Ok(Json(CalendarListResponse {
        calendars: summaries,
        total,
    }))
}

async fn list_calendars_from_db(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Calendar>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteCalendarRepository::new(pool.clone());
            Ok(repo.list_by_user(user_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresCalendarRepository::new(pool.clone());
            Ok(repo.list_by_user(user_id).await?)
        }
    }
}

// --- K-049: GET /api/v1/calendars/:id ---

#[derive(Serialize, specta::Type)]
pub struct CalendarDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    #[specta(type = f64)]
    pub created_at: i64,
    #[specta(type = f64)]
    pub updated_at: i64,
}

pub async fn get_calendar(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(calendar_id): Path<Uuid>,
) -> Result<Json<CalendarDetail>, KestrelError> {
    let cal = find_calendar_from_db(&state, calendar_id).await?;
    let cal = cal.ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;

    // Verify ownership through account
    verify_calendar_ownership(&state, user_id, &cal).await?;

    Ok(Json(CalendarDetail {
        id: cal.id.0,
        account_id: cal.account_id.0,
        external_id: cal.external_id,
        name: cal.name,
        color: cal.color,
        is_primary: cal.is_primary,
        created_at: cal.created_at,
        updated_at: cal.updated_at,
    }))
}

async fn find_calendar_from_db(
    state: &AppState,
    calendar_id: Uuid,
) -> Result<Option<Calendar>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteCalendarRepository::new(pool.clone());
            Ok(repo.find_by_id(calendar_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresCalendarRepository::new(pool.clone());
            Ok(repo.find_by_id(calendar_id).await?)
        }
    }
}

// --- K-050: GET /api/v1/events ---

#[derive(Deserialize, specta::Type)]
pub struct EventListParams {
    pub calendar_id: Option<Uuid>,
    #[serde(default = "default_start_time")]
    #[specta(type = f64)]
    pub start_time: i64,
    #[serde(default = "default_end_time")]
    #[specta(type = f64)]
    pub end_time: i64,
}

fn default_start_time() -> i64 {
    Utc::now().timestamp()
}

fn default_end_time() -> i64 {
    Utc::now().timestamp() + 30 * 24 * 60 * 60 // 30 days from now
}

#[derive(Serialize, specta::Type)]
pub struct EventSummary {
    pub id: Uuid,
    pub occurrence_id: String,
    pub account_id: Uuid,
    pub calendar_id: Uuid,
    pub external_id: String,
    pub title: String,
    pub description: Option<String>,
    pub organizer_email: Option<String>,
    pub organizer_name: Option<String>,
    pub attendees: Option<String>,
    pub rrule: Option<String>,
    pub calendar_color: Option<String>,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
    pub is_all_day: bool,
    pub location: Option<String>,
    pub status: Option<String>,
    pub has_conflict: bool,
}

#[derive(Serialize, specta::Type)]
pub struct EventListResponse {
    pub events: Vec<EventSummary>,
    #[specta(type = f64)]
    pub total: usize,
}

pub async fn list_events(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Query(params): Query<EventListParams>,
) -> Result<Json<EventListResponse>, KestrelError> {
    let events = list_events_from_db(
        &state,
        user_id,
        params.calendar_id,
        params.start_time,
        params.end_time,
    )
    .await?;

    let colors: std::collections::HashMap<Uuid, Option<String>> =
        list_calendars_from_db(&state, user_id)
            .await?
            .into_iter()
            .map(|c| (c.id.0, c.color))
            .collect();

    let summaries: Vec<EventSummary> = events
        .into_iter()
        .flat_map(|e| {
            let color = colors.get(&e.calendar_id.0).cloned().flatten();
            event_occurrences(&e, params.start_time, params.end_time)
                .into_iter()
                .map(|(start, end)| EventSummary {
                    id: e.id.0,
                    occurrence_id: format!("{}:{}", e.id.0, start),
                    account_id: e.account_id.0,
                    calendar_id: e.calendar_id.0,
                    external_id: e.external_id.clone(),
                    title: e.title.clone(),
                    description: e.description.clone(),
                    organizer_email: e.organizer_email.clone(),
                    organizer_name: e.organizer_name.clone(),
                    attendees: e.attendees.clone(),
                    rrule: e.recurrence_rules.clone(),
                    calendar_color: color.clone(),
                    start_time: start,
                    end_time: end,
                    is_all_day: e.is_all_day,
                    location: e.location.clone(),
                    status: e.status.clone(),
                    has_conflict: e.has_conflict,
                })
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(Json(EventListResponse {
        total: summaries.len(),
        events: summaries,
    }))
}

fn overlaps(start: i64, end: i64, window_start: i64, window_end: i64) -> bool {
    end >= window_start && start <= window_end
}

/// Expand one event into the occurrences that intersect the requested window.
///
/// ponytail: expansion treats the stored start timestamp as UTC wall clock.
/// Rules carrying a TZID are parsed by `rrule`, but a bare FREQ rule expands in
/// UTC, so DST shifts need the provider to supply `DTSTART;TZID=...`.
fn event_occurrences(event: &CalendarEvent, window_start: i64, window_end: i64) -> Vec<(i64, i64)> {
    let duration = (event.end_time - event.start_time).max(0);
    let base = (event.start_time, event.start_time + duration);
    let non_recurring = || {
        if overlaps(base.0, base.1, window_start, window_end) {
            vec![base]
        } else {
            Vec::new()
        }
    };

    let Some(rule) = event
        .recurrence_rules
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty() && r.to_ascii_uppercase().contains("FREQ="))
    else {
        return non_recurring();
    };

    let (Some(dt_start), Some(dt_end)) = (
        rrule::Tz::UTC.timestamp_opt(event.start_time, 0).single(),
        rrule::Tz::UTC.timestamp_opt(window_end, 0).single(),
    ) else {
        return non_recurring();
    };

    let Ok(set) = rrule::RRuleSet::new(dt_start).set_from_string(rule) else {
        return non_recurring();
    };

    set.before(dt_end)
        .all_unchecked()
        .into_iter()
        .map(|occurrence| {
            let start = occurrence.timestamp();
            (start, start + duration)
        })
        .filter(|(start, end)| overlaps(*start, *end, window_start, window_end))
        .collect()
}

async fn list_events_from_db(
    state: &AppState,
    user_id: Uuid,
    calendar_id: Option<Uuid>,
    start_time: i64,
    end_time: i64,
) -> Result<Vec<CalendarEvent>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteEventRepository::new(pool.clone());
            Ok(repo
                .list_range(user_id, start_time, end_time, calendar_id)
                .await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresEventRepository::new(pool.clone());
            Ok(repo
                .list_range(user_id, start_time, end_time, calendar_id)
                .await?)
        }
    }
}

// --- K-051: GET /api/v1/events/:id ---

#[derive(Serialize, specta::Type)]
pub struct EventDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub calendar_id: Uuid,
    pub external_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
    pub is_all_day: bool,
    pub recurrence_rules: Option<String>,
    pub organizer_email: Option<String>,
    pub organizer_name: Option<String>,
    pub attendees: Option<String>,
    pub status: Option<String>,
    #[specta(type = f64)]
    pub created_at: i64,
    #[specta(type = f64)]
    pub updated_at: i64,
}

pub async fn get_event(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<Json<EventDetail>, KestrelError> {
    let event = find_event_from_db(&state, event_id).await?;
    let event = event.ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;

    // Verify ownership through account
    verify_event_ownership(&state, user_id, &event).await?;

    Ok(Json(EventDetail {
        id: event.id.0,
        account_id: event.account_id.0,
        calendar_id: event.calendar_id.0,
        external_id: event.external_id,
        title: event.title,
        description: event.description,
        location: event.location,
        start_time: event.start_time,
        end_time: event.end_time,
        is_all_day: event.is_all_day,
        recurrence_rules: event.recurrence_rules,
        organizer_email: event.organizer_email,
        organizer_name: event.organizer_name,
        attendees: event.attendees,
        status: event.status,
        created_at: event.created_at,
        updated_at: event.updated_at,
    }))
}

pub(crate) async fn find_event_from_db(
    state: &AppState,
    event_id: Uuid,
) -> Result<Option<CalendarEvent>, KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteEventRepository::new(pool.clone());
            Ok(repo.find_by_id(event_id).await?)
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresEventRepository::new(pool.clone());
            Ok(repo.find_by_id(event_id).await?)
        }
    }
}

// --- K-052: POST /api/v1/events ---

#[derive(Deserialize, specta::Type)]
pub struct CreateEventRequest {
    pub calendar_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
    pub is_all_day: bool,
    pub recurrence_rules: Option<String>,
    pub attendees: Option<String>,
}

#[derive(Serialize, specta::Type)]
pub struct CreateEventResponse {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub title: String,
    #[specta(type = f64)]
    pub start_time: i64,
    #[specta(type = f64)]
    pub end_time: i64,
    #[specta(type = f64)]
    pub created_at: i64,
}

pub async fn create_event(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<CreateEventResponse>), KestrelError> {
    if body.title.is_empty() {
        return Err(KestrelError::BadRequest("Title is required".to_string()));
    }

    if body.start_time >= body.end_time {
        return Err(KestrelError::BadRequest(
            "start_time must be before end_time".to_string(),
        ));
    }

    // Verify calendar belongs to user
    let cal = find_calendar_from_db(&state, body.calendar_id).await?;
    let cal = cal.ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;
    verify_calendar_ownership(&state, user_id, &cal).await?;

    let account = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(cal.account_id.0)
            .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(cal.account_id.0)
            .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    let now = Utc::now().timestamp();
    let event_id = Uuid::new_v4();
    let external_id = format!("local-{}", event_id);

    let payload = crate::plugins::traits::EventPayload {
        id: event_id.to_string(),
        external_id: external_id.clone(),
        title: body.title.clone(),
        description: body.description.clone(),
        location: body.location.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
        is_all_day: body.is_all_day,
        recurrence_rules: body.recurrence_rules.clone(),
        organizer_email: None,
        organizer_name: None,
        attendees: body.attendees.clone(),
        status: Some("confirmed".to_string()),
    };

    let plugin_manager = state.plugin_manager.read().await;
    if let Some(plugin) = plugin_manager.find_by_id(&account.provider)
        && let Err(e) = plugin
            .as_calendar_provider()
            .mutate_event(&auth_token, "create", &payload)
            .await
    {
        tracing::warn!("Upstream calendar plugin error on create: {:?}", e);
    }

    let event = CalendarEvent {
        id: DbUuid::new(event_id),
        account_id: cal.account_id,
        calendar_id: DbUuid(body.calendar_id),
        external_id,
        title: body.title,
        description: body.description,
        location: body.location,
        start_time: body.start_time,
        end_time: body.end_time,
        is_all_day: body.is_all_day,
        recurrence_rules: body.recurrence_rules,
        organizer_email: None,
        organizer_name: None,
        attendees: body.attendees,
        status: Some("confirmed".to_string()),
        has_conflict: false,
        created_at: now,
        updated_at: now,
    };

    upsert_event_to_db(&state, &event).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateEventResponse {
            id: event.id.0,
            calendar_id: event.calendar_id.0,
            title: event.title,
            start_time: event.start_time,
            end_time: event.end_time,
            created_at: event.created_at,
        }),
    ))
}

async fn upsert_event_to_db(state: &AppState, event: &CalendarEvent) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteEventRepository::new(pool.clone());
            repo.upsert(event).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresEventRepository::new(pool.clone());
            repo.upsert(event).await?;
        }
    }
    Ok(())
}

// --- K-053: PATCH /api/v1/events/:id ---

#[derive(Deserialize, specta::Type)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub location: Option<Option<String>>,
    #[specta(type = Option<f64>)]
    pub start_time: Option<i64>,
    #[specta(type = Option<f64>)]
    pub end_time: Option<i64>,
    pub is_all_day: Option<bool>,
    pub recurrence_rules: Option<Option<String>>,
    pub attendees: Option<Option<String>>,
    pub status: Option<String>,
}

pub async fn update_event(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(event_id): Path<Uuid>,
    Json(body): Json<UpdateEventRequest>,
) -> Result<Json<EventDetail>, KestrelError> {
    let event = find_event_from_db(&state, event_id).await?;
    let mut event = event.ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;

    verify_event_ownership(&state, user_id, &event).await?;

    if let Some(title) = body.title {
        event.title = title;
    }
    if let Some(description) = body.description {
        event.description = description;
    }
    if let Some(location) = body.location {
        event.location = location;
    }
    if let Some(start_time) = body.start_time {
        event.start_time = start_time;
    }
    if let Some(end_time) = body.end_time {
        event.end_time = end_time;
    }
    if let Some(is_all_day) = body.is_all_day {
        event.is_all_day = is_all_day;
    }
    if let Some(recurrence_rules) = body.recurrence_rules {
        event.recurrence_rules = recurrence_rules;
    }
    if let Some(attendees) = body.attendees {
        event.attendees = attendees;
    }
    if let Some(status) = body.status {
        event.status = Some(status);
    }
    event.updated_at = Utc::now().timestamp();

    let account = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(event.account_id.0)
            .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(event.account_id.0)
            .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    let payload = crate::plugins::traits::EventPayload {
        id: event.id.0.to_string(),
        external_id: event.external_id.clone(),
        title: event.title.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        start_time: event.start_time,
        end_time: event.end_time,
        is_all_day: event.is_all_day,
        recurrence_rules: event.recurrence_rules.clone(),
        organizer_email: event.organizer_email.clone(),
        organizer_name: event.organizer_name.clone(),
        attendees: event.attendees.clone(),
        status: event.status.clone(),
    };

    let plugin_manager = state.plugin_manager.read().await;
    if let Some(plugin) = plugin_manager.find_by_id(&account.provider)
        && let Err(e) = plugin
            .as_calendar_provider()
            .mutate_event(&auth_token, "update", &payload)
            .await
    {
        tracing::warn!("Upstream calendar plugin error on update: {:?}", e);
    }

    upsert_event_to_db(&state, &event).await?;

    Ok(Json(EventDetail {
        id: event.id.0,
        account_id: event.account_id.0,
        calendar_id: event.calendar_id.0,
        external_id: event.external_id,
        title: event.title,
        description: event.description,
        location: event.location,
        start_time: event.start_time,
        end_time: event.end_time,
        is_all_day: event.is_all_day,
        recurrence_rules: event.recurrence_rules,
        organizer_email: event.organizer_email,
        organizer_name: event.organizer_name,
        attendees: event.attendees,
        status: event.status,
        created_at: event.created_at,
        updated_at: event.updated_at,
    }))
}

// --- K-225: POST /api/v1/events/rsvp_external ---

#[derive(Deserialize)]
pub struct RsvpExternalRequest {
    pub external_id: String,
    pub status: String,
}

pub async fn rsvp_external(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Json(body): Json<RsvpExternalRequest>,
) -> Result<Json<EventDetail>, KestrelError> {
    // In a real scenario we might need to know the account, but we iterate available accounts for the user.
    let accounts = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_user_id(user_id)
            .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_user_id(user_id)
            .await?
        }
    };

    let mut found_event = None;
    for acc in accounts {
        let repo: Box<dyn crate::core::repository::EventRepository> = match &state.db {
            crate::db::pool::DbPool::Sqlite(pool) => Box::new(
                crate::db::sqlite::event_repository::SqliteEventRepository::new(pool.clone()),
            ),
            crate::db::pool::DbPool::Postgres(pool) => Box::new(
                crate::db::postgres::event_repository::PostgresEventRepository::new(pool.clone()),
            ),
        };

        if let Ok(Some(ev)) = repo.find_by_external_id(acc.id.0, &body.external_id).await {
            found_event = Some((ev, acc));
            break;
        }
    }

    let (mut event, _account) =
        found_event.ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;

    verify_event_ownership(&state, user_id, &event).await?;

    event.status = Some(body.status.clone());
    event.updated_at = Utc::now().timestamp();

    upsert_event_to_db(&state, &event).await?;

    Ok(Json(EventDetail {
        id: event.id.0,
        account_id: event.account_id.0,
        calendar_id: event.calendar_id.0,
        external_id: event.external_id,
        title: event.title,
        description: event.description,
        location: event.location,
        start_time: event.start_time,
        end_time: event.end_time,
        is_all_day: event.is_all_day,
        recurrence_rules: event.recurrence_rules,
        organizer_email: event.organizer_email,
        organizer_name: event.organizer_name,
        attendees: event.attendees,
        status: event.status,
        created_at: event.created_at,
        updated_at: event.updated_at,
    }))
}

// --- K-054: DELETE /api/v1/events/:id ---

pub async fn delete_event(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(event_id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    let event = find_event_from_db(&state, event_id).await?;
    let event = event.ok_or_else(|| KestrelError::NotFound("Event not found".to_string()))?;

    verify_event_ownership(&state, user_id, &event).await?;

    let account = match &state.db {
        crate::db::pool::DbPool::Sqlite(pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(event.account_id.0)
            .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(event.account_id.0)
            .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    let plugin_manager = state.plugin_manager.read().await;
    if let Some(plugin) = plugin_manager.find_by_id(&account.provider)
        && let Err(e) = plugin
            .as_calendar_provider()
            .delete_event(&auth_token, &event.external_id)
            .await
    {
        tracing::warn!("Upstream calendar plugin error on delete: {:?}", e);
    }

    soft_delete_event_from_db(&state, event_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn soft_delete_event_from_db(state: &AppState, event_id: Uuid) -> Result<(), KestrelError> {
    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteEventRepository::new(pool.clone());
            repo.soft_delete(event_id).await?;
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresEventRepository::new(pool.clone());
            repo.soft_delete(event_id).await?;
        }
    }
    Ok(())
}

// --- Ownership verification helpers ---

pub(crate) async fn verify_account_ownership(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<bool, KestrelError> {
    use crate::core::repository::AccountRepository;
    use crate::db::postgres::account_repository::PostgresAccountRepository;
    use crate::db::sqlite::account_repository::SqliteAccountRepository;

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteAccountRepository::new(pool.clone(), state.vault_secret.clone())
                .with_fallback_key(state.vault_old_secret.clone());
            let account = repo.find_by_id(account_id).await?;
            Ok(account.map(|a| *a.user_id == user_id).unwrap_or(false))
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone(), state.vault_secret.clone())
                .with_fallback_key(state.vault_old_secret.clone());
            let account = repo.find_by_id(account_id).await?;
            Ok(account.map(|a| *a.user_id == user_id).unwrap_or(false))
        }
    }
}

async fn verify_calendar_ownership(
    state: &AppState,
    user_id: Uuid,
    calendar: &Calendar,
) -> Result<(), KestrelError> {
    let owns = verify_account_ownership(state, user_id, calendar.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Calendar not found".to_string()));
    }
    Ok(())
}

pub(crate) async fn verify_event_ownership(
    state: &AppState,
    user_id: Uuid,
    event: &CalendarEvent,
) -> Result<(), KestrelError> {
    let owns = verify_account_ownership(state, user_id, event.account_id.0).await?;
    if !owns {
        return Err(KestrelError::NotFound("Event not found".to_string()));
    }
    Ok(())
}

// --- K-1597: Calendar Feeds ---

#[derive(Deserialize)]
pub struct FeedQuery {
    pub token: String,
}

pub fn escape_ics(s: &str) -> String {
    s.replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
        .replace("\r", "")
}

pub fn format_ics_datetime(ts: i64, is_all_day: bool) -> String {
    if is_all_day {
        chrono::Utc
            .timestamp_opt(ts, 0)
            .unwrap()
            .format("%Y%m%d")
            .to_string()
    } else {
        chrono::Utc
            .timestamp_opt(ts, 0)
            .unwrap()
            .format("%Y%m%dT%H%M%SZ")
            .to_string()
    }
}

pub async fn get_calendar_feed(
    State(state): State<AppState>,
    Path(filename): Path<String>,
    Query(query): Query<FeedQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, KestrelError> {
    let raw_token = query.token;

    let id_str = filename.strip_suffix(".ics").unwrap_or(&filename);
    let id = Uuid::parse_str(id_str)
        .map_err(|_| KestrelError::BadRequest("Invalid calendar ID".to_string()))?;

    // Hash token
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let token_hash = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // Look up calendar by ID and token hash
    let calendar = match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteCalendarRepository::new(pool.clone())
                .get_calendar_by_feed_token(id, &token_hash)
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresCalendarRepository::new(pool.clone())
                .get_calendar_by_feed_token(id, &token_hash)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Calendar feed not found".to_string()))?;

    let account = match state.db {
        DbPool::Sqlite(ref pool) => {
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(calendar.account_id.0)
            .await?
        }
        DbPool::Postgres(ref pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(
                pool.clone(),
                state.vault_secret.clone(),
            )
            .with_fallback_key(state.vault_old_secret.clone())
            .find_by_id(calendar.account_id.0)
            .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".to_string()))?;

    let user_id = account.user_id.0;

    let now = chrono::Utc::now().timestamp();
    let window_start = now - (30 * 24 * 60 * 60);
    let window_end = now + (365 * 24 * 60 * 60);

    // Get events for the calendar within the window
    let events = match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteEventRepository::new(pool.clone())
                .list_range(user_id, window_start, window_end, Some(calendar.id.0))
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresEventRepository::new(pool.clone())
                .list_range(user_id, window_start, window_end, Some(calendar.id.0))
                .await?
        }
    };

    let mut ics = String::new();
    ics.push_str("BEGIN:VCALENDAR\r\n");
    ics.push_str("VERSION:2.0\r\n");
    ics.push_str("PRODID:-//Kestrel//NONSGML v1.0//EN\r\n");
    ics.push_str("CALSCALE:GREGORIAN\r\n");
    let cal_name = escape_ics(&calendar.name);
    ics.push_str(&format!("X-WR-CALNAME:{cal_name}\r\n"));

    let mut max_updated_at = calendar.updated_at;

    for event in events {
        max_updated_at = max_updated_at.max(event.updated_at);
        let occurrences = event_occurrences(&event, window_start, window_end);
        for (start_ts, end_ts) in occurrences {
            ics.push_str("BEGIN:VEVENT\r\n");

            // Stable UID
            let uid = if event.recurrence_rules.is_some() {
                format!("{}-{}@kestrel", event.id.0, start_ts)
            } else {
                format!("{}@kestrel", event.id.0)
            };
            ics.push_str(&format!("UID:{uid}\r\n"));

            ics.push_str(&format!(
                "DTSTAMP:{}\r\n",
                format_ics_datetime(event.created_at, false)
            ));

            if event.is_all_day {
                ics.push_str(&format!(
                    "DTSTART;VALUE=DATE:{}\r\n",
                    format_ics_datetime(start_ts, true)
                ));
                // In all-day, DTEND is exclusive and generally the next day
                ics.push_str(&format!(
                    "DTEND;VALUE=DATE:{}\r\n",
                    format_ics_datetime(end_ts, true)
                ));
            } else {
                ics.push_str(&format!(
                    "DTSTART:{}\r\n",
                    format_ics_datetime(start_ts, false)
                ));
                ics.push_str(&format!("DTEND:{}\r\n", format_ics_datetime(end_ts, false)));
            }

            ics.push_str(&format!("SUMMARY:{}\r\n", escape_ics(&event.title)));

            if let Some(desc) = &event.description {
                ics.push_str(&format!("DESCRIPTION:{}\r\n", escape_ics(desc)));
            }

            if let Some(loc) = &event.location {
                ics.push_str(&format!("LOCATION:{}\r\n", escape_ics(loc)));
            }

            if let Some(status) = &event.status {
                ics.push_str(&format!("STATUS:{}\r\n", escape_ics(status)));
            }

            ics.push_str("END:VEVENT\r\n");
        }
    }
    ics.push_str("END:VCALENDAR\r\n");

    let mut hasher = Sha256::new();
    hasher.update(ics.as_bytes());
    let etag = format!("\"{}\"", URL_SAFE_NO_PAD.encode(hasher.finalize()));

    if headers.get(IF_NONE_MATCH).is_some_and(|h| h.to_str().is_ok_and(|s| s == etag)) {
        return Ok((StatusCode::NOT_MODIFIED, [(ETAG, etag)]).into_response());
    }

    let last_modified = chrono::Utc
        .timestamp_opt(max_updated_at, 0)
        .unwrap()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, "text/calendar; charset=utf-8".to_string()),
            (ETAG, etag),
            (LAST_MODIFIED, last_modified),
        ],
        ics,
    )
        .into_response())
}

#[derive(Serialize, specta::Type)]
pub struct FeedResponse {
    pub url: String,
    pub token: String,
}

pub async fn create_calendar_feed(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<FeedResponse>, KestrelError> {
    // Check if calendar exists and belongs to user
    let cal = match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteCalendarRepository::new(pool.clone())
                .find_by_id(id)
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresCalendarRepository::new(pool.clone())
                .find_by_id(id)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;

    // Verify ownership
    verify_calendar_ownership(&state, user_id, &cal).await?;

    // Generate random token
    let mut token_bytes = [0u8; 32]; // 256 bits of entropy
    OsRng.fill_bytes(&mut token_bytes);
    let token = URL_SAFE_NO_PAD.encode(token_bytes);

    // Hash token
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = URL_SAFE_NO_PAD.encode(hasher.finalize());

    // Upsert feed token in repository
    match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteCalendarRepository::new(pool.clone())
                .upsert_feed_token(id, user_id, &token_hash)
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresCalendarRepository::new(pool.clone())
                .upsert_feed_token(id, user_id, &token_hash)
                .await?
        }
    };

    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("localhost:3000");

    let base_url = std::env::var("KESTREL_BASE_URL").unwrap_or_else(|_| format!("http://{}", host));

    let url = format!("{}/feeds/calendars/{}.ics?token={}", base_url, id, token);

    Ok(Json(FeedResponse { url, token }))
}

pub async fn revoke_calendar_feed(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, KestrelError> {
    // Check if calendar exists and belongs to user
    let cal = match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteCalendarRepository::new(pool.clone())
                .find_by_id(id)
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresCalendarRepository::new(pool.clone())
                .find_by_id(id)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Calendar not found".to_string()))?;

    // Verify ownership
    verify_calendar_ownership(&state, user_id, &cal).await?;

    match state.db {
        DbPool::Sqlite(ref pool) => {
            SqliteCalendarRepository::new(pool.clone())
                .delete_feed_token(id)
                .await?
        }
        DbPool::Postgres(ref pool) => {
            PostgresCalendarRepository::new(pool.clone())
                .delete_feed_token(id)
                .await?
        }
    };

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_ics() {
        assert_eq!(escape_ics("Hello World"), "Hello World");
        assert_eq!(escape_ics("Line 1\nLine 2"), "Line 1\\nLine 2");
        assert_eq!(escape_ics("Comma,;slash\\"), "Comma\\,\\;slash\\\\");
    }

    #[test]
    fn test_format_ics_datetime() {
        let ts = chrono::Utc
            .with_ymd_and_hms(2023, 10, 28, 14, 30, 45)
            .unwrap()
            .timestamp();
        assert_eq!(format_ics_datetime(ts, false), "20231028T143045Z");
        assert_eq!(format_ics_datetime(ts, true), "20231028");
    }
    use crate::core::types::DbUuid;

    fn event(start_time: i64, end_time: i64, rrule: Option<&str>) -> CalendarEvent {
        CalendarEvent {
            id: DbUuid::from(Uuid::new_v4()),
            account_id: DbUuid::from(Uuid::new_v4()),
            calendar_id: DbUuid::from(Uuid::new_v4()),
            external_id: "ext".to_string(),
            title: "Standup".to_string(),
            description: None,
            location: None,
            start_time,
            end_time,
            is_all_day: false,
            recurrence_rules: rrule.map(str::to_string),
            organizer_email: None,
            organizer_name: None,
            attendees: None,
            status: None,
            has_conflict: false,
            created_at: start_time,
            updated_at: start_time,
        }
    }

    #[test]
    fn weekly_rule_expands_into_window() {
        let start = 1_700_000_000;
        let ev = event(start, start + 1800, Some("FREQ=WEEKLY;COUNT=4"));

        let occurrences = event_occurrences(&ev, start - 86400, start + 14 * 86400);

        assert_eq!(occurrences.len(), 3);
        assert_eq!(occurrences[0], (start, start + 1800));
        assert_eq!(occurrences[1].0 - start, 7 * 86400);
        assert_eq!(occurrences[2].0 - start, 14 * 86400);
    }

    #[test]
    fn non_recurring_event_outside_window_is_dropped() {
        let start = 1_700_000_000;
        let ev = event(start, start + 1800, None);

        assert!(event_occurrences(&ev, start + 86400, start + 2 * 86400).is_empty());
        assert_eq!(
            event_occurrences(&ev, start - 60, start + 60),
            vec![(start, start + 1800)]
        );
    }

    #[test]
    fn malformed_rule_falls_back_to_single_occurrence() {
        let start = 1_700_000_000;
        let ev = event(start, start + 1800, Some("not a rule"));

        assert_eq!(
            event_occurrences(&ev, start - 60, start + 60),
            vec![(start, start + 1800)]
        );
    }
}
