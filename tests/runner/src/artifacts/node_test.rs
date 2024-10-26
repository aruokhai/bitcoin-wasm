use exports::component::node::types::{BitcoinNetwork, NodeConfig, SocketAddress};
use std::env;
use std::path::PathBuf;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{self, WasiHttpCtx, WasiHttpView};

include!(concat!(env!("OUT_DIR"), "/node_WIT.rs"));


pub fn test_node(){
    
    let (nodeworld, mut store ,node) = create_node().unwrap();
    let wallet_filter = "0014622d0e3b6cc7af423cc297fd931a9528e8548292".to_string();
    nodeworld.component_node_types().client_node().call_add_filter(&mut store, node.clone(), &wallet_filter).unwrap().unwrap();
    let balance = nodeworld.component_node_types().client_node().call_get_balance(&mut store, node.clone()).unwrap().unwrap();
    assert_eq!(balance, 10_0000_0000);


}

fn create_node() -> wasmtime::Result<(Nodeworld, Store<ServerWasiView>, ResourceAny)> {
    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(false);
    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    let pathtowasm  = PathBuf::from(env::var_os("OUT_DIR").unwrap())
            .join(format!("node-composed.wasm"));

    // Add the command world (aka WASI CLI) to the linker
    wasmtime_wasi::add_to_linker_sync(&mut linker).unwrap();
    wasmtime_wasi_http::add_only_http_to_linker_sync(&mut linker).unwrap();
    
    let wasi_view = ServerWasiView::new();
    let mut store = Store::new(&engine, wasi_view);
    
    let component = Component::from_file(&engine, pathtowasm).unwrap();
    // linker.define_unknown_imports_as_traps(&component).unwrap();
    let instance =  Nodeworld::instantiate(&mut store, &component, &linker)
        .unwrap();
    
    let ip_config = SocketAddress{ ip: "127.0.0.1".to_string(), port: 19444 };
    let network_config = BitcoinNetwork::Regtest;
    let wallet_address = "bcrt1qvgksuwmvc7h5y0xzjl7exx549r59fq5jgcdm93".to_string();
    let wallet_filter = "0014622d0e3b6cc7af423cc297fd931a9528e8548292".to_string();
    let genesis_blockhash = "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206".to_string();

    let node_config = NodeConfig{ socket_address: ip_config, network: network_config, wallet_address, genesis_blockhash};
    let resource = instance.component_node_types().client_node().call_constructor(&mut store, &node_config).unwrap();
    
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
            .preopened_dir("/tmp", ".", DirPerms::all(), FilePerms::all()).unwrap()
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