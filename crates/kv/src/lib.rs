#[allow(warnings)]
mod bindings;
mod clock;
mod bit_cask_key;
mod config;
mod merge_config;
mod field_generator;
mod entry;
mod segment;
mod store;
mod errors;
mod segments;
mod key_directory;
mod merged_state;
mod kvstore;

use std::{cell::RefCell, sync::Arc};
use bit_cask_key::{UUIDWasiKey, UUIDWasiKeyFrom};
use clock::WasiClock;
use config::Config;
use kvstore::KVStore as HashKVStore;
use bindings::exports::component::kv::types::{Error, Guest, GuestKvstore};
use merge_config::MergeConfig;
use store::WasiStore;

struct Component;

struct KVStore {
    inner: RefCell<HashKVStore<UUIDWasiKey, WasiStore>>,
}

impl GuestKvstore for KVStore {
    fn insert(&self, key: String, value: Vec<u8>) -> Result<(), Error> {
        return  self.inner.borrow_mut().update(UUIDWasiKey::from(key), value).map_err(|err| err.into());
    }

    fn get(&self, key: String) -> Result<Vec<u8>, Error> {
        return  self.inner.borrow_mut().get(UUIDWasiKey::from(key)).map_err(|err| err.into());
    }

    fn delete(&self, key: String) -> Result<(), Error> {
        return self.inner.borrow_mut().delete(UUIDWasiKey::from(key)).map_err(|err| err.into());
    }
    
    fn new() -> Self {
        let merge_config = MergeConfig::new_with_all_segments_to_read(UUIDWasiKeyFrom);
        let config  = Config::new("bitcoin-wasm".to_string(), 1048576, 1024, Some(merge_config), Arc::new(WasiClock{}));
        let hashtree = HashKVStore::new(&config).unwrap();
        Self{ inner:  RefCell::new(hashtree)}
    }
}

impl Guest for Component {
    
    type Kvstore = KVStore;
}

bindings::export!(Component with_types_in bindings);