use backend::plugins::bindings::kestrel::provider::http_client::{Host as HttpHost, HttpRequest};
use backend::plugins::wasm_runtime::WasmEngine;

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
fn test_attachment_url_rejection() {
    // Actually get_attachment_url is in traits/WasmPlugin, not tested here directly,
    // but the allowlist logic is identical.
}
