
use exports::component::kv::types::{Kvstore, Error};
use std::env;
use std::path::PathBuf;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{self, WasiHttpCtx, WasiHttpView};

include!(concat!(env!("OUT_DIR"), "/kv_WIT.rs"));


pub fn test_store(){
    
    let (storeworld, mut store ,kvstore) = create_kvstore().unwrap();

    let mut key_value = vec![];
    for i in 31..40 {
        let k = i * 3;
        key_value.push((i.to_string(), b"fgkjfgkjgsioureghiuvhngurihuirhdgihiutgurghjfkhgjkhfgbkjhfghfksjghhfgjkhsjk".to_vec()))
    }

    for (key,value) in key_value.iter() {
        storeworld.component_kv_types().kvstore().call_insert(&mut store, kvstore.clone(),key, value).unwrap().unwrap();
    }
    for (key,value) in key_value.iter() {
        let searched_value = storeworld.component_kv_types().kvstore().call_get(&mut store, kvstore.clone(), key).unwrap().unwrap();
        assert_eq!(searched_value, value.to_owned());
        println!("working motherfucker")
    }


}

fn create_kvstore() -> wasmtime::Result<(Kvworld, Store<ServerWasiView>, ResourceAny)> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    let pathtowasm  = PathBuf::from(env::var_os("OUT_DIR").unwrap())
            .join(format!("wasm32-wasi/debug/kv.wasm"));

    // Add the command world (aka WASI CLI) to the linker
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    wasmtime_wasi_http::add_only_http_to_linker_sync(&mut linker).unwrap();
    
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);
    
    let component = Component::from_file(&engine, pathtowasm).unwrap();
    // linker.define_unknown_imports_as_traps(&component).unwrap();
    let instance =  Kvworld::instantiate(&mut store, &component, &linker)
        .unwrap();
    

    let resource = instance.component_kv_types().kvstore().call_constructor(&mut store).unwrap();
    
    wasmtime::Result::Ok((instance, store, resource))
}


struct ServerWasiView {
    table: ResourceTable,
    ctx: WasiCtx,
    http_ctx: WasiHttpCtx,
}

impl ServerWasiView {
    fn new() -> Self {
        let table = ResourceTable::new();
        let http_ctx = WasiHttpCtx::new();
        let ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir("./testfolder", ".", DirPerms::all(), FilePerms::all()).unwrap()
            .inherit_network()
            .allow_ip_name_lookup(true)
            .allow_tcp(true)
            .build();

        Self { table, ctx, http_ctx }
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

impl WasiHttpView for ServerWasiView {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }
}

// pub fn test_store(){
//     test_insert();
//     test_delete();
// } 


//  fn test_insert() {
    
// }

//  fn test_delete() {
//     let mut key_value:  Vec<(String,String)> = vec![];
//     for i in 1..20 {
//         let k = i * 3;
//         key_value.push((i.to_string(), k.to_string()))
//     }

//     let new_store = Store::new();
//     for (key,value) in key_value.iter() {
//         new_store.insert(&KeyValuePair{ key: key.to_owned(), value: value.to_owned()}).unwrap();
//     }
//     for (key,value) in key_value.iter() {
//         new_store.delete(key).unwrap();
//         assert!(matches!(new_store.search(key), Err(_)))
//     }
// }

