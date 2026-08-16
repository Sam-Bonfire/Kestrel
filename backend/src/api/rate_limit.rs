use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{ConnectInfo, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tokio::sync::Mutex;

/// Fixed-window rate limiter using in-memory counters.
///
/// Each window is `window_duration` long. Once a key exceeds `max_requests`
/// within a window, subsequent requests are rejected with 429.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RateLimiterInner>,
}

struct RateLimiterInner {
    max_requests: u64,
    window_duration: std::time::Duration,
    counters: Mutex<HashMap<String, WindowEntry>>,
}

struct WindowEntry {
    count: u64,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u64, window_duration: std::time::Duration) -> Self {
        Self {
            inner: Arc::new(RateLimiterInner {
                max_requests,
                window_duration,
                counters: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Check if the given key is allowed. Returns Ok(()) if allowed,
    /// Err(remaining) with the retry-after duration if rate-limited.
    async fn check(&self, key: &str) -> Result<(), std::time::Duration> {
        let mut counters = self.inner.counters.lock().await;
        let now = Instant::now();

        let entry = counters
            .entry(key.to_string())
            .or_insert_with(|| WindowEntry {
                count: 0,
                window_start: now,
            });

        // If the window has expired, reset it
        if now.duration_since(entry.window_start) >= self.inner.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        if entry.count > self.inner.max_requests {
            let elapsed = now.duration_since(entry.window_start);
            let retry_after = self.inner.window_duration.saturating_sub(elapsed);
            Err(retry_after)
        } else {
            Ok(())
        }
    }
}

/// Extract the client IP from the request.
fn extract_client_ip(req: &Request<Body>) -> String {
    // Try X-Forwarded-For header first (for reverse proxies)
    if let Some(first_ip) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|f| f.split(',').next())
    {
        return first_ip.trim().to_string();
    }

    // Try connecting info (direct connections)
    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<IpAddr>>() {
        return addr.to_string();
    }

    // Fallback to unknown
    "unknown".to_string()
}

/// Auth-specific rate limit middleware: max 10 requests per minute per IP.
pub async fn auth_rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);
    let key = format!("auth:{}", client_ip);

    match limiter.check(&key).await {
        Ok(()) => next.run(req).await,
        Err(retry_after) => {
            let retry_secs = retry_after.as_secs().max(1);
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "Rate limit exceeded. Too many requests.",
                    "retry_after_secs": retry_secs,
                })),
            )
                .into_response();
            response
                .headers_mut()
                .insert("Retry-After", retry_secs.to_string().parse().unwrap());
            response
        }
    }
}

/// General rate limit middleware: max 100 requests per minute per IP.
pub async fn general_rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let client_ip = extract_client_ip(&req);
    let key = format!("general:{}", client_ip);

    match limiter.check(&key).await {
        Ok(()) => next.run(req).await,
        Err(retry_after) => {
            let retry_secs = retry_after.as_secs().max(1);
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "Rate limit exceeded. Too many requests.",
                    "retry_after_secs": retry_secs,
                })),
            )
                .into_response();
            response
                .headers_mut()
                .insert("Retry-After", retry_secs.to_string().parse().unwrap());
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3, std::time::Duration::from_secs(60));

        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_rejects_over_limit() {
        let limiter = RateLimiter::new(2, std::time::Duration::from_secs(60));

        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_err());
    }

    #[tokio::test]
    async fn test_rate_limiter_separate_keys() {
        let limiter = RateLimiter::new(1, std::time::Duration::from_secs(60));

        assert!(limiter.check("key-a").await.is_ok());
        assert!(limiter.check("key-b").await.is_ok());
        assert!(limiter.check("key-a").await.is_err());
        assert!(limiter.check("key-b").await.is_err());
    }
}
