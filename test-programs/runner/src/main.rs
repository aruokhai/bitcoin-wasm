use std::env;
use std::path::PathBuf;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

wasmtime::component::bindgen!({
    path: "wit/artifacts.wit",
    world: "artifacts",
    async: false
});

fn main() {
    // run_test();
    let _ = run_test();
}


pub fn run_test() -> wasmtime::Result<()> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    let pathtowasm  = PathBuf::from(env::var_os("OUT_DIR").unwrap())
            .join(format!("artifactsartifacts.wasm"));

    // Add the command world (aka WASI CLI) to the linker
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);

    let component = Component::from_file(&engine, pathtowasm).unwrap();
    let (instance, _) = Artifacts::instantiate(&mut store, &component, &linker)
        .unwrap();
    instance
        .call_test_store(&mut store)
        
}


struct ServerWasiView {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = ResourceTable::new();
        let ctx = WasiCtxBuilder::new().inherit_stdio().preopened_dir("/tmp", ".", DirPerms::all(), FilePerms::all()).unwrap().build();

        Self { table, ctx }
    }
}

impl WasiView for ServerWasiView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
