#[allow(warnings)]
mod bindings;

use std::{borrow::BorrowMut, cell::RefCell};

use bindings::{Btree, Guest};
use btree::BTree;
use error::Error;
use bindings::exports::component::store::types::{GuestBtree, KeyValuePair as KV};
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

struct Store {
    inner: RefCell<BTree>,
}

impl GuestBtree for Store {
    fn insert(&self, kv: KV) -> Result<(), bindings::exports::component::store::types::Error> {
        let key_value_pair = KeyValuePair::from(kv);
        return  self.inner.borrow_mut().insert(key_value_pair).map_err(|err| err.into());
    }

    fn search(&self, key: bindings::exports::component::store::types::Key) -> Result<KeyValuePair, bindings::exports::component::store::types::Error> {
        todo!()
    }

    fn delete(&self, key: bindings::exports::component::store::types::Key) -> Result<(), bindings::exports::component::store::types::Error> {
        todo!()
    }
}

impl Guest for Component {

    fn create_store() -> Result<BTree, Error> {
        return btree::BTreeBuilder::new().build();
    }

   
    


}

bindings::export!(Component with_types_in bindings);
