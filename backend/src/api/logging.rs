use std::time::Instant;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

/// K-060: Request logging middleware with request ID generation.
///
/// - Generates a unique X-Request-Id per request (UUID v4)
/// - Logs: method, path, status code, duration, body size
/// - Adds X-Request-Id header to the response
pub async fn request_logging_middleware(
    State(_state): State<super::router::AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = Instant::now();

    // Log request body size if available
    let (parts, body) = req.into_parts();
    let (body_size, body) = {
        let size: usize = 0;
        // We can't easily peek at body size without consuming it,
        // so we'll log it from the response instead
        let body = axum::body::Body::new(body);
        (size, body)
    };
    let _ = body_size;

    // Reconstruct request with body
    let req = Request::from_parts(parts, body);

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        "Request started"
    );

    let mut response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    // Extract response body size from Content-Length header if available
    let response_body_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);

    // Add request ID to response headers
    if let Ok(val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("X-Request-Id", val);
    }

    if status.is_server_error() {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration.as_millis() as u64,
            response_body_size = response_body_size,
            "Request completed with error"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = status.as_u16(),
            duration_ms = duration.as_millis() as u64,
            response_body_size = response_body_size,
            "Request completed"
        );
    }

    response
}
