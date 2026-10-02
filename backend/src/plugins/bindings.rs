use wasmtime::component::bindgen;

bindgen!({
    world: "kestrel-plugin",
    path: "../wit",
    imports: { default: async },
    exports: { default: async },
});
