// use anyhow::Context;
// use std::path::PathBuf;
// use wasmtime::component::*;
// use wasmtime::{Config, Engine, Store};
// use wasmtime_wasi::preview2::{command, WasiCtx, WasiCtxBuilder, WasiView};

// wasmtime::component::bindgen!({
//     path: "wit/world.wit",
//     world: "example",
//     async: false
// });

fn main() {
    // run_test();
    println!("hello");
}


// pub fn run_test() -> wasmtime::Result<i32> {
//     let mut config = Config::default();
//     config.wasm_component_model(true);
//     config.async_support(true);
//     let engine = Engine::new(&config)?;
//     let mut linker = Linker::new(&engine);

//     // Add the command world (aka WASI CLI) to the linker
//     command::add_to_linker(&mut linker).context("Failed to link command world")?;
//     let wasi_view = ServerWasiView::new();
//     let mut store = Store::new(&engine, wasi_view);

//     let component = Component::from_file(&engine, path).context("Component file not found")?;

//     let (instance, _) = Example::instantiate_async(&mut store, &component, &linker)
//         .await
//         .context("Failed to instantiate the example world")?;
//     instance
//         .call_test_store()
//         .await
//         .context("Failed to call add function")
// }

// struct ServerWasiView {
//     table: ResourceTable,
//     ctx: WasiCtx,
// }

// impl ServerWasiView {
//     fn new() -> Self {
//         let table = ResourceTable::new();
//         let ctx = WasiCtxBuilder::new().inherit_stdio().build();

//         Self { table, ctx }
//     }
// }

// impl WasiView for ServerWasiView {
//     fn table(&mut self) -> &mut ResourceTable {
//         &mut self.table
//     }

//     fn ctx(&mut self) -> &mut WasiCtx {
//         &mut self.ctx
//     }
// }