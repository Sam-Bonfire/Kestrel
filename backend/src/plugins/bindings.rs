use wasmtime::component::bindgen;

bindgen!({
    world: "kestrel-plugin",
    path: "../wit",
    async: true
});
