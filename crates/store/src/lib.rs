#[allow(warnings)]
mod bindings;

use std::{borrow::BorrowMut, cell::RefCell};

use btree::BTree;
use bindings::exports::component::store::types::{Guest, GuestStore, KeyValuePair as KV};
use node_type::KeyValuePair;

struct Component;

mod node_type;
mod page_layout;
mod page;
mod node;
mod error;
mod btree;
mod pager;
mod wal;

struct KVStore {
    inner: RefCell<BTree>,
}

impl GuestStore for KVStore {
    fn insert(&self, kv: KV) -> Result<(), bindings::exports::component::store::types::Error> {
        let key_value_pair = KeyValuePair::from(kv);
        return  self.inner.borrow_mut().insert(key_value_pair).map_err(|err| err.into());
    }

    fn search(&self, key: bindings::exports::component::store::types::Key) -> Result<bindings::exports::component::store::types::KeyValuePair, bindings::exports::component::store::types::Error> {
        return self.inner.borrow_mut().search(key).map_err(|err| err.into()).map(KeyValuePair::into);
    }

    fn delete(&self, key: bindings::exports::component::store::types::Key) -> Result<(), bindings::exports::component::store::types::Error> {
        return self.inner.borrow_mut().delete(key).map_err(|err| err.into());
    }
    
    fn new() -> Self {
        let btree = btree::BTreeBuilder::new().build().unwrap();
        Self{ inner:  RefCell::new(btree)}
    }
}

impl Guest for Component {
    
    type Store  = KVStore;
}

bindings::export!(Component with_types_in bindings);
