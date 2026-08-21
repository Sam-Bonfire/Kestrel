use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize)]
pub struct CalendarSummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct CalendarListResponse {
    pub calendars: Vec<CalendarSummary>,
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

#[derive(Serialize)]
pub struct CalendarDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub external_id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
    pub created_at: i64,
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

#[derive(Deserialize)]
pub struct EventListParams {
    pub calendar_id: Option<Uuid>,
    #[serde(default = "default_start_time")]
    pub start_time: i64,
    #[serde(default = "default_end_time")]
    pub end_time: i64,
}

fn default_start_time() -> i64 {
    Utc::now().timestamp()
}

fn default_end_time() -> i64 {
    Utc::now().timestamp() + 30 * 24 * 60 * 60 // 30 days from now
}

#[derive(Serialize)]
pub struct EventSummary {
    pub id: Uuid,
    pub account_id: Uuid,
    pub calendar_id: Uuid,
    pub external_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub is_all_day: bool,
    pub status: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize)]
pub struct EventListResponse {
    pub events: Vec<EventSummary>,
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
        params.start_time,
        params.end_time,
        params.calendar_id,
    )
    .await?;
    let total = events.len();

    let summaries: Vec<EventSummary> = events
        .into_iter()
        .map(|e| EventSummary {
            id: e.id.0,
            account_id: e.account_id.0,
            calendar_id: e.calendar_id.0,
            external_id: e.external_id,
            title: e.title,
            description: e.description,
            location: e.location,
            start_time: e.start_time,
            end_time: e.end_time,
            is_all_day: e.is_all_day,
            status: e.status,
            created_at: e.created_at,
            updated_at: e.updated_at,
        })
        .collect();

    Ok(Json(EventListResponse {
        events: summaries,
        total,
    }))
}

async fn list_events_from_db(
    state: &AppState,
    user_id: Uuid,
    start_time: i64,
    end_time: i64,
    calendar_id: Option<Uuid>,
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

#[derive(Serialize)]
pub struct EventDetail {
    pub id: Uuid,
    pub account_id: Uuid,
    pub calendar_id: Uuid,
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
    pub created_at: i64,
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

async fn find_event_from_db(
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

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub calendar_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub is_all_day: bool,
    pub recurrence_rules: Option<String>,
    pub attendees: Option<String>,
}

#[derive(Serialize)]
pub struct CreateEventResponse {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub title: String,
    pub start_time: i64,
    pub end_time: i64,
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
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .find_by_id(cal.account_id.0)
                .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
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
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    plugin
        .as_calendar_provider()
        .mutate_event(&auth_token, "create", &payload)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin error: {:?}",
                e
            ))))
        })?;

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

#[derive(Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub location: Option<Option<String>>,
    pub start_time: Option<i64>,
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
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .find_by_id(event.account_id.0)
                .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
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
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    plugin
        .as_calendar_provider()
        .mutate_event(&auth_token, "update", &payload)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin error: {:?}",
                e
            ))))
        })?;

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
            crate::db::sqlite::account_repository::SqliteAccountRepository::new(pool.clone())
                .find_by_id(event.account_id.0)
                .await?
        }
        crate::db::pool::DbPool::Postgres(pool) => {
            crate::db::postgres::account_repository::PostgresAccountRepository::new(pool.clone())
                .find_by_id(event.account_id.0)
                .await?
        }
    }
    .ok_or_else(|| KestrelError::NotFound("Account not found".into()))?;

    let auth_token = account.access_token.ok_or_else(|| {
        KestrelError::BadRequest("Account is missing an access token".to_string())
    })?;

    let plugin_manager = state.plugin_manager.read().await;
    let plugin = plugin_manager
        .find_by_id(&account.provider)
        .ok_or_else(|| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin {} not loaded",
                account.provider
            ))))
        })?;

    plugin
        .as_calendar_provider()
        .delete_event(&auth_token, &event.external_id)
        .await
        .map_err(|e| {
            KestrelError::Internal(Box::new(crate::core::error::SimpleError(format!(
                "Plugin error: {:?}",
                e
            ))))
        })?;

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

async fn verify_account_ownership(
    state: &AppState,
    user_id: Uuid,
    account_id: Uuid,
) -> Result<bool, KestrelError> {
    use crate::core::repository::AccountRepository;
    use crate::db::postgres::account_repository::PostgresAccountRepository;
    use crate::db::sqlite::account_repository::SqliteAccountRepository;

    match &state.db {
        DbPool::Sqlite(pool) => {
            let repo = SqliteAccountRepository::new(pool.clone());
            let account = repo.find_by_id(account_id).await?;
            Ok(account.map(|a| *a.user_id == user_id).unwrap_or(false))
        }
        DbPool::Postgres(pool) => {
            let repo = PostgresAccountRepository::new(pool.clone());
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

async fn verify_event_ownership(
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
