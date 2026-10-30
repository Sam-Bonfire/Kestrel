use backend::plugins::bindings::kestrel::provider::http_client::{Host as HttpHost, HttpRequest};
use backend::plugins::wasm_runtime::{WasmEngine, ensure_public_host, is_blocked_ip};
use std::net::IpAddr;

#[tokio::test]
async fn test_http_allowlist_rejection() {
    let engine = WasmEngine::new().expect("Failed to create WasmEngine");
    let mut store = engine
        .create_store(vec!["gmail.googleapis.com".to_string()])
        .expect("Failed to create store");
    let state = store.data_mut();

    let invalid_hosts = vec![
        "https://169.254.169.254/latest/meta-data",
        "https://localhost:8080/api",
        "https://10.0.0.1/admin",
        "https://evil.com/phish",
        "http://gmail.googleapis.com", // non-https
    ];

    for url in invalid_hosts {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: vec![],
            body: None,
        };
        let res = state.request(req).await;
        assert!(
            res.is_err(),
            "Request to {} should be rejected, but it succeeded: {:?}",
            url,
            res
        );
    }
}

#[tokio::test]
async fn test_http_allowlist_acceptance() {
    let engine = WasmEngine::new().expect("Failed to create WasmEngine");
    let mut store = engine
        .create_store(vec![
            "gmail.googleapis.com".to_string(),
            "www.googleapis.com".to_string(),
        ])
        .expect("Failed to create store");
    let state = store.data_mut();

    let req = HttpRequest {
        method: "GET".to_string(),
        url: "https://gmail.googleapis.com/generate_204".to_string(), // public valid endpoint
        headers: vec![],
        body: None,
    };

    let res = state.request(req).await;
    match res {
        Err(e) if e.contains("is not in the allowlist") => {
            panic!("Valid URL was rejected by allowlist: {}", e)
        }
        _ => {}
    }
}

#[test]
fn blocked_ip_ranges_cover_loopback_private_and_metadata() {
    for ip in [
        "127.0.0.1",
        "127.1.2.3",
        "10.1.2.3",
        "172.16.0.9",
        "192.168.1.1",
        "169.254.169.254",
        "0.0.0.0",
        "255.255.255.255",
        "240.0.0.1",
        "::1",
        "fc00::1",
        "fe80::1",
        "::ffff:127.0.0.1",
    ] {
        assert!(
            is_blocked_ip(ip.parse::<IpAddr>().unwrap()),
            "{ip} should be blocked"
        );
    }

    for ip in ["8.8.8.8", "1.1.1.1", "142.250.1.1", "2606:4700::1111"] {
        assert!(
            !is_blocked_ip(ip.parse::<IpAddr>().unwrap()),
            "{ip} should be allowed"
        );
    }
}

#[tokio::test]
async fn blocked_literal_hosts_are_rejected_without_a_request() {
    for host in ["127.0.0.1", "169.254.169.254", "10.0.0.1", "192.168.0.1"] {
        let err = ensure_public_host(host).await.expect_err(host);
        assert!(err.contains("blocked address"), "{host}: {err}");
    }

    // Public literals need no DNS, so this stays offline.
    ensure_public_host("8.8.8.8")
        .await
        .expect("public IP is allowed");
}
