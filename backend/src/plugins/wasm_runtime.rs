use wasmtime::{Config, Engine};
use wasmtime::component::Linker;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, ResourceTable};

pub struct WasmState {
    pub wasi: WasiCtx,
    pub table: ResourceTable,
}

impl WasiView for WasmState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
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
        config.async_support(true);
        
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        
        // Add WASI to the linker
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        
        crate::plugins::bindings::KestrelPlugin::add_to_linker(&mut linker, |state| state)?;

        Ok(Self { engine, linker })
    }

    pub fn create_store(&self) -> wasmtime::Store<WasmState> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let state = WasmState {
            wasi,
            table: ResourceTable::new(),
        };

        wasmtime::Store::new(&self.engine, state)
    }
}

use crate::plugins::bindings::ClientCredentials;
use crate::plugins::bindings::kestrel::provider::http_client::{Host as HttpHost, HttpRequest, HttpResponse};

#[async_trait::async_trait]
impl crate::plugins::bindings::KestrelPluginImports for WasmState {
    async fn get_client_credentials(&mut self, provider: String) -> Result<ClientCredentials, String> {
        let provider_upper = provider.to_uppercase();
        let client_id = std::env::var(format!("{}_CLIENT_ID", provider_upper))
            .map_err(|_| format!("{} client ID not configured in environment", provider))?;
        let client_secret = std::env::var(format!("{}_CLIENT_SECRET", provider_upper))
            .map_err(|_| format!("{} client secret not configured in environment", provider))?;
            
        Ok(ClientCredentials {
            client_id,
            client_secret,
        })
    }
}

#[async_trait::async_trait]
impl HttpHost for WasmState {
    async fn request(&mut self, req: HttpRequest) -> Result<HttpResponse, String> {
        let client = reqwest::Client::new();
        
        let method = match req.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => return Err(format!("Unsupported HTTP method: {}", req.method)),
        };
        
        let mut builder = client.request(method, &req.url);
        for (k, v) in req.headers {
            builder = builder.header(k, v);
        }
        
        if let Some(body) = req.body {
            builder = builder.body(body);
        }
        
        let res = builder.send().await;
        let res = match res {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        
        let status = res.status().as_u16();
        let mut headers = Vec::new();
        for (k, v) in res.headers() {
            if let Ok(v_str) = v.to_str() {
                headers.push((k.as_str().to_string(), v_str.to_string()));
            }
        }
        
        let body_bytes = res.bytes().await.unwrap_or_default().to_vec();
        
        Ok(HttpResponse {
            status,
            headers,
            body: body_bytes,
        })
    }
}
