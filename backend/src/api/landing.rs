use super::router::AppState;
use axum::Router;
use tower_http::services::ServeDir;

/// Static landing page (portfolio + downloads) served by the backend itself.
///
/// Looks for the built `frontend-landing/dist` directory, overridable via the
/// `LANDING_DIR` env var. When the directory is missing (e.g. API-only
/// deployments), returns an empty router and unknown routes 404 as usual.
pub fn landing_fallback() -> Router<AppState> {
    let dir = std::env::var("LANDING_DIR").unwrap_or_else(|_| "frontend-landing/dist".to_string());
    if !std::path::Path::new(&dir).join("index.html").exists() {
        tracing::info!(
            "Landing page not built ({} missing), skipping static fallback",
            dir
        );
        return Router::new();
    }
    tracing::info!("Serving landing page from {}", dir);
    Router::new().fallback_service(ServeDir::new(dir).append_index_html_on_directories(true))
}
