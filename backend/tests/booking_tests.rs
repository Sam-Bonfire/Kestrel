mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use backend::api::rate_limit::RateLimiter;
use backend::api::router::{AppState, create_router};
use backend::plugins::manager::PluginManager;
use common::{seed_account, seed_calendar, seed_event, setup_test_db};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tower::ServiceExt;
use uuid::Uuid;

async fn create_test_state() -> AppState {
    let db = setup_test_db().await;
    let plugin_manager = Arc::new(RwLock::new(PluginManager::new()));
    let (sync_tx, _) = broadcast::channel(100);
    let (sync_job_tx, _) = tokio::sync::mpsc::channel(100);
    AppState {
        db,
        jwt_secret: common::TEST_SECRET.to_string(),
        plugin_manager,
        sync_tx,
        sync_job_tx,
        auth_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
        general_rate_limiter: RateLimiter::new(1000, Duration::from_secs(60)),
    }
}

async fn register(app: &axum::Router, username: &str) -> (Uuid, String) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username": username, "password": "Password123!"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg: Value = serde_json::from_slice(&body).unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/token")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username": username, "password": "Password123!"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tok: Value = serde_json::from_slice(&body).unwrap();
    (
        reg["user_id"].as_str().unwrap().parse().unwrap(),
        tok["token"].as_str().unwrap().to_string(),
    )
}

async fn authed(
    app: &axum::Router,
    token: &str,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    let req_body = Body::from(body.map(|b| b.to_string()).unwrap_or_default());
    let res = app
        .clone()
        .oneshot(builder.body(req_body).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

async fn public(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let req_body = Body::from(body.map(|b| b.to_string()).unwrap_or_default());
    let res = app
        .clone()
        .oneshot(builder.body(req_body).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, json)
}

async fn setup_page(app: &axum::Router, token: &str, state: &AppState, user_id: Uuid) -> Value {
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let cal = seed_calendar(&state.db, account.id.0, "Work", true).await;
    let (status, page) = authed(
        app,
        token,
        "POST",
        "/api/v1/booking-pages",
        Some(json!({
            "calendar_id": cal.id.0,
            "name": "Coffee chat",
            "duration_mins": 60,
            "buffer_mins": 0,
            "window_days": 7
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    page
}

#[tokio::test]
async fn owner_crud_roundtrip() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner1").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let cal = seed_calendar(&state.db, account.id.0, "Work", true).await;

    let (status, page) = authed(
        &app,
        &token,
        "POST",
        "/api/v1/booking-pages",
        Some(json!({
            "calendar_id": cal.id.0, "name": "Coffee", "duration_mins": 30
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let id = page["id"].as_str().unwrap().to_string();
    let slug = page["slug"].as_str().unwrap().to_string();
    assert_eq!(page["buffer_mins"], 0);
    assert_eq!(page["window_days"], 14);

    let (status, got) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/booking-pages/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["slug"].as_str().unwrap(), slug);

    let (status, _) = authed(
        &app,
        &token,
        "PATCH",
        &format!("/api/v1/booking-pages/{id}"),
        Some(json!({"name": "Deep chat", "is_active": false})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, rotated) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/booking-pages/{id}/rotate"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(rotated["slug"].as_str().unwrap(), slug);

    let (status, _) = authed(
        &app,
        &token,
        "DELETE",
        &format!("/api/v1/booking-pages/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = authed(
        &app,
        &token,
        "GET",
        &format!("/api/v1/booking-pages/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn validation_and_ownership() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner2").await;
    let (_, other_token) = register(&app, "other2").await;
    let account = seed_account(&state.db, user_id, "gmail", "Work").await;
    let cal = seed_calendar(&state.db, account.id.0, "Work", true).await;

    for body in [
        json!({"calendar_id": cal.id.0, "name": "", "duration_mins": 30}),
        json!({"calendar_id": cal.id.0, "name": "x", "duration_mins": 3}),
        json!({"calendar_id": cal.id.0, "name": "x", "duration_mins": 30, "buffer_mins": 999}),
    ] {
        let (status, _) = authed(&app, &token, "POST", "/api/v1/booking-pages", Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let (status, _) = authed(
        &app,
        &token,
        "POST",
        "/api/v1/booking-pages",
        Some(json!({"calendar_id": Uuid::new_v4(), "name": "x", "duration_mins": 30})),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let page = setup_page(&app, &token, &state, user_id).await;
    let id = page["id"].as_str().unwrap();
    let (status, _) = authed(
        &app,
        other_token.as_str(),
        "GET",
        &format!("/api/v1/booking-pages/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn public_slots_skip_busy_and_booking_conflicts() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner3").await;
    let page = setup_page(&app, &token, &state, user_id).await;
    let slug = page["slug"].as_str().unwrap().to_string();

    let now = chrono::Utc::now().timestamp();
    let busy_start = now + 3600 - (now % 900);
    let account_id: Uuid = seed_account(&state.db, user_id, "gmail", "W").await.id.0;
    let cal_id: Uuid = page["calendar_id"].as_str().unwrap().parse().unwrap();
    seed_event(
        &state.db,
        account_id,
        cal_id,
        "Busy",
        busy_start,
        busy_start + 3600,
    )
    .await;

    let (status, info) = public(&app, "GET", &format!("/api/book/{slug}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(info["duration_mins"], 60);

    let (status, slots) = public(
        &app,
        "GET",
        &format!("/api/book/{slug}/slots?from={now}&to={}", now + 86400),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let arr = slots["slots"].as_array().unwrap();
    assert!(!arr.is_empty());
    assert!(
        !arr.iter().any(|s| {
            let (a, b) = (
                s["start_time"].as_i64().unwrap(),
                s["end_time"].as_i64().unwrap(),
            );
            a < busy_start + 3600 && busy_start < b
        }),
        "busy hour must not appear"
    );

    let first = arr[0]["start_time"].as_i64().unwrap();
    let (status, booked) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(json!({"guest_name": "Ada", "guest_email": "ada@example.com", "start_time": first})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(booked["start_time"], first);

    // Same slot again -> conflict.
    let (status, _) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(json!({"guest_name": "Bob", "guest_email": "bob@example.com", "start_time": first})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Overlapping off-grid slot -> conflict.
    let (status, _) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(json!({"guest_name": "Cid", "guest_email": "cid@example.com", "start_time": first + 1800})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Bad email + unknown slug.
    let (status, _) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(json!({"guest_name": "E", "guest_email": "nope", "start_time": first + 7200})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = public(&app, "GET", "/api/book/deadbeef", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deactivated_page_hides_and_guest_html_renders() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner4").await;
    let page = setup_page(&app, &token, &state, user_id).await;
    let slug = page["slug"].as_str().unwrap().to_string();
    let id = page["id"].as_str().unwrap().to_string();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/book/{slug}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let ct = res.headers()["content-type"].to_str().unwrap().to_string();
    assert!(ct.contains("text/html"));

    let (status, _) = authed(
        &app,
        &token,
        "PATCH",
        &format!("/api/v1/booking-pages/{id}"),
        Some(json!({"is_active": false})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = public(&app, "GET", &format!("/api/book/{slug}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = public(
        &app,
        "GET",
        &format!("/api/book/{slug}/slots?from=1&to=2"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(
            json!({"guest_name": "Z", "guest_email": "z@example.com", "start_time": 1_800_000_000}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn rotate_kills_old_slug() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner6").await;
    let page = setup_page(&app, &token, &state, user_id).await;
    let old = page["slug"].as_str().unwrap().to_string();
    let id = page["id"].as_str().unwrap().to_string();
    let (status, rotated) = authed(
        &app,
        &token,
        "POST",
        &format!("/api/v1/booking-pages/{id}/rotate"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let new = rotated["slug"].as_str().unwrap().to_string();
    assert_ne!(old, new);
    let (status, _) = public(&app, "GET", &format!("/api/book/{old}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = public(&app, "GET", &format!("/api/book/{new}"), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn cannot_create_page_on_foreign_calendar() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, _) = register(&app, "owner7a").await;
    let (_, other_token) = register(&app, "owner7b").await;
    let account = seed_account(&state.db, user_id, "gmail", "W").await;
    let cal = seed_calendar(&state.db, account.id.0, "W", true).await;
    let (status, _) = authed(
        &app,
        &other_token,
        "POST",
        "/api/v1/booking-pages",
        Some(json!({"calendar_id": cal.id.0, "name": "Squat", "duration_mins": 30})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn buffer_and_grid_enforced_at_book_time() {
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner8").await;
    let account = seed_account(&state.db, user_id, "gmail", "W").await;
    let cal = seed_calendar(&state.db, account.id.0, "W", true).await;
    let (status, page) = authed(
        &app,
        &token,
        "POST",
        "/api/v1/booking-pages",
        Some(json!({
            "calendar_id": cal.id.0, "name": "Buffered",
            "duration_mins": 60, "buffer_mins": 60, "window_days": 7
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let slug = page["slug"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().timestamp();
    let (status, slots) = public(
        &app,
        "GET",
        &format!("/api/book/{slug}/slots?from={now}&to={}", now + 86400),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let first = slots["slots"][0]["start_time"].as_i64().unwrap();

    let book = |start: i64| {
        let app = app.clone();
        let slug = slug.clone();
        async move {
            public(
                &app,
                "POST",
                &format!("/api/book/{slug}"),
                Some(
                    json!({"guest_name": "G", "guest_email": "g@example.com", "start_time": start}),
                ),
            )
            .await
            .0
        }
    };
    assert_eq!(book(first).await, StatusCode::CREATED);
    // Adjacent slot violates the 60-min buffer.
    assert_eq!(book(first + 3600).await, StatusCode::CONFLICT);
    // Off-grid start is rejected, not silently accepted.
    assert_eq!(book(first + 3600 + 1).await, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn guest_name_with_quotes_stays_valid_json() {
    use backend::core::repository::EventRepository;
    use backend::db::sqlite::event_repository::SqliteEventRepository;
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, token) = register(&app, "owner9").await;
    let page = setup_page(&app, &token, &state, user_id).await;
    let slug = page["slug"].as_str().unwrap().to_string();
    let now = chrono::Utc::now().timestamp();
    let (status, slots) = public(
        &app,
        "GET",
        &format!("/api/book/{slug}/slots?from={now}&to={}", now + 86400),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let first = slots["slots"][0]["start_time"].as_i64().unwrap();
    let (status, booked) = public(
        &app,
        "POST",
        &format!("/api/book/{slug}"),
        Some(json!({
            "guest_name": "O\"Brien <boss>",
            "guest_email": "ob@example.com",
            "start_time": first
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let pool = common::get_sqlite_pool(&state.db);
    let repo = SqliteEventRepository::new(pool);
    let event_id: Uuid = booked["event_id"].as_str().unwrap().parse().unwrap();
    let event = repo.find_by_id(event_id).await.unwrap().unwrap();
    let attendees: Value = serde_json::from_str(event.attendees.as_deref().unwrap()).unwrap();
    assert_eq!(attendees[0]["email"], "ob@example.com");
    assert_eq!(attendees[0]["name"], "O\"Brien boss");
}

#[tokio::test]
async fn concurrent_inserts_yield_single_winner() {
    use backend::core::models::CalendarEvent;
    use backend::core::repository::BookingPageRepository;
    use backend::core::types::DbUuid;
    use backend::db::sqlite::booking_repository::SqliteBookingRepository;
    let state = create_test_state().await;
    let app = create_router(state.clone());
    let (user_id, _) = register(&app, "owner10").await;
    let account = seed_account(&state.db, user_id, "gmail", "W").await;
    let cal = seed_calendar(&state.db, account.id.0, "W", true).await;
    let pool = common::get_sqlite_pool(&state.db);
    let base = 1_800_000_000i64;
    let futs: Vec<_> = (0..5)
        .map(|i| {
            let repo = SqliteBookingRepository::new(pool.clone());
            let ev = CalendarEvent {
                id: DbUuid::from(Uuid::new_v4()),
                account_id: account.id,
                calendar_id: cal.id,
                external_id: format!("race-{i}"),
                title: "Race".to_string(),
                description: None,
                location: None,
                start_time: base,
                end_time: base + 3600,
                is_all_day: false,
                recurrence_rules: None,
                organizer_email: None,
                organizer_name: None,
                attendees: None,
                status: Some("confirmed".to_string()),
                has_conflict: false,
                created_at: base,
                updated_at: base,
            };
            async move { repo.insert_event_if_free(&ev, base, base + 3600).await }
        })
        .collect();
    let outs = futures::future::join_all(futs).await;
    assert_eq!(
        outs.iter().filter(|r| matches!(r, Ok(true))).count(),
        1,
        "exactly one concurrent insert must win"
    );
}
