use std::env;
use std::path::PathBuf;
use artifacts::node_test::test_node;
use artifacts::store_test::test_store;
use wasmtime::component::*;
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{self, WasiHttpCtx, WasiHttpView};

mod artifacts;

fn main() {
    test_node();
    //test_store();
}

