#[allow(warnings)]
mod bindings;
use std::{cell::RefCell};

use node::Node;
use bindings::exports::component::node::types::{Guest, GuestClientNode, NodeConfig};
use bindings::component::kv::types::{Kvstore };

mod node;
mod p2p;
mod tcpsocket;
mod util;
mod messages;
mod chain;
mod db;
struct Component;

struct BitcoinNode {
    inner: RefCell<Node>,
}

impl GuestClientNode for BitcoinNode {
    fn get_balance(&self) -> Result<i64, u32> {
        return  self.inner.borrow_mut().balance().map_err(|err| err.to_error_code());
    }

    fn add_filter(&self, filter: String) -> Result<(), u32> {
        return  self.inner.borrow_mut().add_filter(filter).map_err(|err| err.to_error_code());
    }

    fn new(config: NodeConfig) -> Self {
        Self{ inner:  Node::new(config.into(), Kvstore::new().into()).into()}
    }
}

impl Guest for Component {
    
    type ClientNode  = BitcoinNode;
   
}

bindings::export!(Component with_types_in bindings);

