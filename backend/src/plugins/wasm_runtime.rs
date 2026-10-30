use std::time::Duration;
use wasmtime::component::Linker;
use wasmtime::{Config, Engine, ResourceLimiter, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

pub struct WasmState {
    pub wasi: WasiCtx,
    pub table: ResourceTable,
    pub limits: StoreLimits,
    pub allowed_hosts: Vec<String>,
}

impl ResourceLimiter for WasmState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        self.limits.memory_growing(current, desired, maximum)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, wasmtime::Error> {
        self.limits.table_growing(current, desired, maximum)
    }
}

impl WasiView for WasmState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

pub struct WasmEngine {
    pub engine: Engine,
    pub linker: Linker<WasmState>,
}

impl WasmEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        // Add WASI to the linker
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        crate::plugins::bindings::KestrelPlugin::add_to_linker::<
            WasmState,
            wasmtime::component::HasSelf<WasmState>,
        >(&mut linker, |state| state)?;

        let engine_clone = engine.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                engine_clone.increment_epoch();
            }
        });

        Ok(Self { engine, linker })
    }

    pub fn create_store(
        &self,
        allowed_hosts: Vec<String>,
    ) -> Result<wasmtime::Store<WasmState>, Box<dyn std::error::Error>> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let limits = StoreLimitsBuilder::new()
            .memory_size(50 * 1024 * 1024)
            .build();

        let state = WasmState {
            wasi,
            table: ResourceTable::new(),
            limits,
            allowed_hosts,
        };

        let mut store = wasmtime::Store::new(&self.engine, state);
        store.limiter(|state| state as &mut dyn ResourceLimiter);
        store.set_fuel(10_000_000)?;
        store.epoch_deadline_trap();
        store.set_epoch_deadline(5);

        Ok(store)
    }
}

use crate::plugins::bindings::ClientCredentials;
use crate::plugins::bindings::kestrel::provider::http_client::{
    Host as HttpHost, HttpRequest, HttpResponse,
};

impl crate::plugins::bindings::KestrelPluginImports for WasmState {
    async fn get_client_credentials(
        &mut self,
        provider: String,
    ) -> Result<ClientCredentials, String> {
        let provider_upper = provider.to_uppercase();
        let alias_upper = match provider_upper.as_str() {
            "GMAIL" => "GOOGLE",
            "GOOGLE" => "GMAIL",
            "OUTLOOK" => "MICROSOFT",
            "MICROSOFT" => "OUTLOOK",
            _ => "",
        };

        let client_id = std::env::var(format!("{}_CLIENT_ID", provider_upper))
            .or_else(|_| {
                if !alias_upper.is_empty() {
                    std::env::var(format!("{}_CLIENT_ID", alias_upper))
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .map_err(|_| format!("{} client ID not configured in environment", provider))?;

        let client_secret = std::env::var(format!("{}_CLIENT_SECRET", provider_upper))
            .or_else(|_| {
                if !alias_upper.is_empty() {
                    std::env::var(format!("{}_CLIENT_SECRET", alias_upper))
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .map_err(|_| format!("{} client secret not configured in environment", provider))?;

        Ok(ClientCredentials {
            client_id,
            client_secret,
        })
    }
}

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;

/// True for addresses a plugin must never reach: loopback, private ranges,
/// link-local (including the cloud metadata address), reserved and unspecified.
pub fn is_blocked_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let first = v4.octets()[0];
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || first == 0
                || first >= 240
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|v4| is_blocked_ip(v4.into()))
        }
    }
}

/// Reject a host whose literal or resolved address is blocked.
///
/// shortcut: only the request host is resolved; redirect hops are re-checked
/// against the allowlist but not re-resolved. Upgrade to a custom
/// `reqwest::dns::Resolve` if a provider ever uses an attacker-controlled name.
pub async fn ensure_public_host(host: &str) -> Result<(), String> {
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return if is_blocked_ip(ip) {
            Err(format!("Host '{}' resolves to a blocked address", host))
        } else {
            Ok(())
        };
    }

    let addrs = tokio::net::lookup_host((host, 443u16))
        .await
        .map_err(|e| format!("DNS lookup failed for '{}': {}", host, e))?;

    let mut resolved = false;
    for addr in addrs {
        resolved = true;
        if is_blocked_ip(addr.ip()) {
            return Err(format!("Host '{}' resolves to a blocked address", host));
        }
    }

    if !resolved {
        return Err(format!("Host '{}' did not resolve", host));
    }

    Ok(())
}

impl HttpHost for WasmState {
    async fn request(&mut self, req: HttpRequest) -> Result<HttpResponse, String> {
        let allowed_hosts = self.allowed_hosts.clone();
        if allowed_hosts.is_empty() {
            return Err("Plugin provider unknown or has no allowed hosts".to_string());
        }

        let parsed_url = reqwest::Url::parse(&req.url).map_err(|e| e.to_string())?;

        if parsed_url.scheme() != "https" {
            return Err("Only HTTPS requests are allowed".to_string());
        }

        let host = parsed_url
            .host_str()
            .ok_or("Invalid host in URL")?
            .to_lowercase();

        let is_allowed = allowed_hosts
            .iter()
            .any(|allowed| host == allowed.to_lowercase());

        if !is_allowed {
            return Err(format!("Host '{}' is not in the allowlist", host));
        }

        ensure_public_host(&host).await?;

        if let Some(body) = &req.body
            && body.len() > MAX_REQUEST_BYTES
        {
            return Err(format!(
                "Request body too large (exceeds {} bytes)",
                MAX_REQUEST_BYTES
            ));
        }

        // redirect loop is handled by reqwest, but we can set redirect policy to custom
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() > 5 {
                    return attempt.error("too many redirects");
                }
                let host = attempt.url().host_str().unwrap_or("").to_lowercase();
                if allowed_hosts
                    .iter()
                    .any(|allowed| host == allowed.to_lowercase())
                {
                    attempt.follow()
                } else {
                    attempt.error("redirect to disallowed host")
                }
            }))
            .build()
            .map_err(|e| e.to_string())?;

        let method = match req.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => return Err(format!("Unsupported HTTP method: {}", req.method)),
        };

        let mut builder = client.request(method, parsed_url);
        for (k, v) in req.headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        let mut res = builder.send().await.map_err(|e| e.to_string())?;

        let status = res.status().as_u16();
        let mut headers = Vec::new();
        for (k, v) in res.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.push((k.as_str().to_string(), v_str.to_string()));
            }
        }

        let mut body_bytes = Vec::new();
        let mut total_size = 0;
        let max_size = 10 * 1024 * 1024; // 10MB

        while let Some(chunk) = res.chunk().await.map_err(|e| e.to_string())? {
            total_size += chunk.len();
            if total_size > max_size {
                return Err("Response body too large (exceeds 10MB)".to_string());
            }
            body_bytes.extend_from_slice(&chunk);
        }

        Ok(HttpResponse {
            status,
            headers,
            body: body_bytes,
        })
    }
}
