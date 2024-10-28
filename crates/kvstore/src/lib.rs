#[allow(warnings)]
mod bindings;

use std::{borrow::BorrowMut, cell::RefCell};

use btree::BTree;
use bindings::exports::component::kvstore::types::{Guest,Error,Key, GuestKvstore, KeyValuePair as KV};
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

impl GuestKvstore for KVStore {
    fn insert(&self, kv: KV) -> Result<(), Error> {
        let key_value_pair = KeyValuePair::from(kv);
        return  self.inner.borrow_mut().insert(key_value_pair).map_err(|err| err.into());
    }

    fn search(&self, key: Key) -> Result<KV, Error> {
        return self.inner.borrow_mut().search(key).map_err(|err| err.into()).map(KeyValuePair::into);
    }

    fn delete(&self, key: Key) -> Result<(), Error> {
        return self.inner.borrow_mut().delete(key).map_err(|err| err.into());
    }
    
    fn new() -> Self {
        let btree = btree::BTreeBuilder::new().build().unwrap();
        Self{ inner:  RefCell::new(btree)}
    }
}

impl Guest for Component {
    
    type Kvstore = KVStore;
}

bindings::export!(Component with_types_in bindings);
