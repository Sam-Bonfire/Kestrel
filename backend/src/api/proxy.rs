use axum::{
    body::Body,
    extract::{Query, State},
    http::{Response, StatusCode, header},
};
use futures::StreamExt;
use serde::Deserialize;
use std::net::IpAddr;
use std::net::SocketAddr;
use url::Url;

use crate::api::router::AppState;
use crate::core::error::KestrelError;

pub struct SafeDnsResolver;

impl reqwest::dns::Resolve for SafeDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let name_str = name.as_str().to_string();
        let fut = async move {
            let addrs = tokio::net::lookup_host(format!("{}:80", name_str)).await?;

            let mut valid_addrs = Vec::new();
            for addr in addrs {
                if is_blocked_ip(&addr.ip()) {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "IP address is blocked",
                    ))
                        as Box<dyn std::error::Error + Send + Sync>);
                }
                valid_addrs.push(addr);
            }

            if valid_addrs.is_empty() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No valid IP addresses found",
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }

            let addrs_iter: Box<dyn Iterator<Item = SocketAddr> + Send> =
                Box::new(valid_addrs.into_iter());
            Ok(addrs_iter)
        };
        Box::pin(fut)
    }
}

const MAX_IMAGE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB

#[derive(Deserialize)]
pub struct ProxyQuery {
    pub url: String,
}

pub async fn proxy_image(
    State(state): State<AppState>,
    Query(query): Query<ProxyQuery>,
) -> Result<Response<Body>, KestrelError> {
    validate_url(&query.url)?;

    // 2. Fetch the image securely via the global client which uses the SafeDnsResolver
    let res = state
        .http_client
        .get(&query.url)
        .send()
        .await
        .map_err(|e| KestrelError::BadRequest(format!("Failed to fetch external image: {}", e)))?;

    if !res.status().is_success() {
        return Err(KestrelError::BadRequest(
            "Failed to fetch external image".to_string(),
        ));
    }

    // 3. Check content type
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !content_type.starts_with("image/") {
        return Err(KestrelError::BadRequest(
            "URL does not point to an image".to_string(),
        ));
    }

    // 4. Check content length (size limit)
    if let Some(length) = res.headers().get(reqwest::header::CONTENT_LENGTH)
        && let Ok(len) = length.to_str().unwrap_or("0").parse::<u64>()
        && len > MAX_IMAGE_SIZE
    {
        return Err(KestrelError::BadRequest("Image too large".to_string()));
    }

    // 5. Build response and stream it securely (DoS protection)
    let mut total_bytes = 0;
    let secure_stream = res.bytes_stream().map(move |result| {
        match result {
            Ok(bytes) => {
                total_bytes += bytes.len() as u64;
                if total_bytes > MAX_IMAGE_SIZE {
                    // Truncate and return error in stream if limits exceeded during download
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Image exceeds size limit",
                    ))
                } else {
                    Ok(bytes)
                }
            }
            Err(e) => Err(std::io::Error::other(e)),
        }
    });

    let body = Body::from_stream(secure_stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(body)
        .map_err(|e| KestrelError::Internal(Box::new(e)))?;

    Ok(response)
}

fn validate_url(url_str: &str) -> Result<(), KestrelError> {
    let url =
        Url::parse(url_str).map_err(|_| KestrelError::BadRequest("Invalid URL".to_string()))?;

    match url.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(KestrelError::BadRequest(
                "Only HTTP/HTTPS allowed".to_string(),
            ));
        }
    }

    // Direct IP bypassing check: if the host is explicitly an IP address, validate it.
    if let Some(host) = url.host_str() {
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_blocked_ip(&ip) {
                return Err(KestrelError::Forbidden(
                    "Direct IP address is blocked".to_string(),
                ));
            }
        } else if host == "localhost" {
            return Err(KestrelError::Forbidden("Localhost is blocked".to_string()));
        }
    } else {
        return Err(KestrelError::BadRequest("Invalid host".to_string()));
    }

    Ok(())
}

fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            ipv4.is_private()
                || ipv4.is_loopback()
                || ipv4.is_link_local()
                || ipv4.is_multicast()
                || ipv4.is_broadcast()
                || ipv4.is_documentation()
                || ipv4.octets() == [169, 254, 169, 254] // AWS metadata
                || ipv4.octets()[0] == 0 // Current network
        }
        IpAddr::V6(ipv6) => {
            if let Some(mapped_ipv4) = ipv6.to_ipv4_mapped() {
                return is_blocked_ip(&IpAddr::V4(mapped_ipv4));
            }
            ipv6.is_loopback() || ipv6.is_multicast()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com/image.png").is_ok());
        assert!(validate_url("http://example.com/image.jpg").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        assert!(matches!(
            validate_url("file:///etc/passwd"),
            Err(KestrelError::BadRequest(_))
        ));
        assert!(matches!(
            validate_url("ftp://example.com/image.png"),
            Err(KestrelError::BadRequest(_))
        ));
        assert!(matches!(
            validate_url("data:image/png;base64,iVBORw0KGgo"),
            Err(KestrelError::BadRequest(_))
        ));
    }

    #[test]
    fn test_validate_url_internal_ip() {
        assert!(matches!(
            validate_url("http://localhost/image.png"),
            Err(KestrelError::BadRequest(_)) | Err(KestrelError::Forbidden(_))
        ));
        assert!(matches!(
            validate_url("http://127.0.0.1/image.png"),
            Err(KestrelError::Forbidden(_))
        ));
        assert!(matches!(
            validate_url("https://10.0.0.1/image.png"),
            Err(KestrelError::Forbidden(_))
        ));
        assert!(matches!(
            validate_url("https://192.168.1.1/image.png"),
            Err(KestrelError::Forbidden(_))
        ));
        // Note: localhost does not parse as IpAddr, so it gets through this check, but SafeDnsResolver will catch it if it resolves to 127.0.0.1.
        // We could also specifically block "localhost".
    }
}
