use std::collections::HashSet;
use std::sync::LazyLock;

use axum::http::HeaderValue;

/// Exact origins that may make credentialed cross-origin calls.
/// Tauri WebViews plus local dev servers; anything else comes from config.
fn builtin_origins() -> impl Iterator<Item = String> {
    [
        "http://tauri.localhost",
        "https://tauri.localhost",
        "tauri://localhost",
        "http://localhost:1420",
        "http://localhost:8080",
        "http://127.0.0.1:1420",
        "http://127.0.0.1:8080",
    ]
    .into_iter()
    .map(str::to_string)
}

/// Reduce a base URL to its `scheme://host[:port]` origin, if parseable.
pub fn origin_of_base_url(base_url: &str) -> Option<String> {
    let base = base_url.trim().trim_end_matches('/');
    let (scheme, rest) = base.split_once("://")?;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let host_port = rest.split('/').next()?.trim();
    if host_port.is_empty() {
        return None;
    }
    Some(format!("{}://{}", scheme, host_port))
}

/// Build the allowlist: builtin app/dev origins + KESTREL_BASE_URL origin
/// + EXTRA_ALLOWED_ORIGINS (comma-separated).
pub fn allowed_origins() -> HashSet<String> {
    allowed_origins_from(
        &std::env::var("KESTREL_BASE_URL").ok(),
        &std::env::var("EXTRA_ALLOWED_ORIGINS").ok(),
    )
}

fn allowed_origins_from(base_url: &Option<String>, extra: &Option<String>) -> HashSet<String> {
    let mut set: HashSet<String> = builtin_origins().collect();
    if let Some(base) = base_url
        && let Some(origin) = origin_of_base_url(base)
    {
        set.insert(origin);
    }
    if let Some(extra) = extra {
        for origin in extra.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            set.insert(origin.to_string());
        }
    }
    set
}

static ALLOWLIST: LazyLock<HashSet<String>> = LazyLock::new(allowed_origins);

pub fn is_origin_allowed(origin: &HeaderValue) -> bool {
    origin
        .to_str()
        .map(|s| ALLOWLIST.contains(s))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_of_base_url() {
        assert_eq!(
            origin_of_base_url("https://kestrel.example.com"),
            Some("https://kestrel.example.com".to_string())
        );
        assert_eq!(
            origin_of_base_url("http://100.64.0.5:8080/api/v1/"),
            Some("http://100.64.0.5:8080".to_string())
        );
        assert_eq!(origin_of_base_url("not a url"), None);
        assert_eq!(origin_of_base_url("ftp://host"), None);
        assert_eq!(origin_of_base_url("https://"), None);
    }

    #[test]
    fn test_allowed_origins_from_combines_sources() {
        let set = allowed_origins_from(
            &Some("https://kestrel.example.com".to_string()),
            &Some("https://app.example.com, http://localhost:3000".to_string()),
        );
        assert!(set.contains("https://kestrel.example.com"));
        assert!(set.contains("https://app.example.com"));
        assert!(set.contains("http://localhost:3000"));
        assert!(set.contains("http://tauri.localhost"));
        assert!(set.contains("tauri://localhost"));
        assert!(!set.contains("https://evil.example.com"));
    }

    #[test]
    fn test_allowed_origins_from_ignores_bad_base() {
        let set = allowed_origins_from(&Some("garbage".to_string()), &None);
        assert!(set.contains("http://tauri.localhost"));
        assert_eq!(set.contains("garbage"), false);
    }
}
