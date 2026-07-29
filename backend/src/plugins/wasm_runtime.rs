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
