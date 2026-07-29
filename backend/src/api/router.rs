use axum::middleware;
use axum::routing::{delete, get, post};
use axum::Router;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::accounts;
use super::auth;
use super::calendars;
use super::health::health_check;
use super::messages;
use super::providers;
use super::rate_limit::RateLimiter;
use super::search;
use super::sync;
use crate::api::sync::SyncEvent;
use crate::plugins::manager::PluginManager;

#[derive(Clone)]
pub struct AppState {
    pub db: crate::db::pool::DbPool,
    pub jwt_secret: String,
    pub plugin_manager: std::sync::Arc<tokio::sync::RwLock<PluginManager>>,
    pub sync_tx: broadcast::Sender<SyncEvent>,
    pub auth_rate_limiter: RateLimiter,
    pub general_rate_limiter: RateLimiter,
}

pub fn create_router(state: AppState) -> Router {
    use axum::http::HeaderValue;
    use tower_http::cors::Any;
    
    let cors = CorsLayer::new()
        .allow_origin(vec![
            "http://localhost:1420".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:1420".parse::<HeaderValue>().unwrap(),
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:5173".parse::<HeaderValue>().unwrap(),
            "tauri://localhost".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods(vec![
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(vec![
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true);

    // Public routes (no auth required)
    let public = Router::new()
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/token", post(auth::token))
        .route("/api/v1/auth/me", get(auth::me).route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        )))
        .route("/api/v1/auth/login", get(auth::login))
        .route("/api/v1/auth/callback", get(auth::callback))
        .route("/api/v1/providers", get(providers::list_providers))
        // Auth-specific rate limit: max 10 requests per minute per IP
        .layer(middleware::from_fn_with_state(
            state.auth_rate_limiter.clone(),
            super::rate_limit::auth_rate_limit_middleware,
        ));

    // Protected routes (auth middleware required)
    let protected = Router::new()
        .route("/api/v1/accounts/{id}", delete(accounts::delete_account))
        .route("/api/v1/messages", get(messages::list_messages))
        .route("/api/v1/messages/{id}", get(messages::get_message))
        .route("/api/v1/messages/{id}/attachments/{filename}/redirect", get(messages::get_attachment_redirect))
        .route("/api/v1/messages/{id}/read", post(messages::mark_read))
        .route("/api/v1/messages/{id}/archive", post(messages::archive_message))
        .route("/api/v1/messages/{id}/trash", post(messages::trash_message))
        .route("/api/v1/messages/{id}/star", post(messages::toggle_star))
        .route("/api/v1/messages/{id}/labels", post(messages::update_labels))
        .route("/api/v1/labels", get(super::labels::list_labels).patch(super::labels::update_label))
        .route("/api/v1/messages/bulk", post(messages::bulk_action))
        .route("/api/v1/messages/send", post(messages::send_message))
        .route("/api/v1/search", get(search::search_messages))
        .route("/api/v1/calendars", get(calendars::list_calendars))
        .route("/api/v1/calendars/{id}", get(calendars::get_calendar))
        .route("/api/v1/events", get(calendars::list_events).post(calendars::create_event))
        .route("/api/v1/events/{id}", get(calendars::get_event).patch(calendars::update_event).delete(calendars::delete_event))
        .route("/api/v1/sync/stream", get(sync::sync_stream))
        .route("/api/v1/sync/trigger", post(sync::trigger_sync))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    public
        .merge(protected)
        // Request logging with X-Request-Id
        .layer(middleware::from_fn_with_state(
            state.clone(),
            super::logging::request_logging_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        // General rate limit: max 100 requests per minute per IP
        .layer(middleware::from_fn_with_state(
            state.general_rate_limiter.clone(),
            super::rate_limit::general_rate_limit_middleware,
        ))
        .layer(cors)
        .with_state(state)
}
