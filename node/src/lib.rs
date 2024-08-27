#[allow(warnings)]
mod bindings;
use std::{cell::RefCell};

use node::Node;
use bindings::exports::component::node::types::{Guest,Error, GuestNode, NodeConfig};




mod node;
mod p2p;
mod tcpsocket;
mod util;
mod wallet;
mod messages;

struct Component;

struct BitcoinNode {
    inner: RefCell<Node>,
}

impl GuestNode for BitcoinNode {
    fn get_balance(&self) -> Result<i64, Error> {
        return  self.inner.borrow_mut().get_balance().map_err(|_| Error::NetworkError);
    }

    
    fn new(config: NodeConfig) -> Self {
        return Self{ inner:  Node::new(config.into()).into()};
    }
}

impl Guest for Component {
    
    type Node  = BitcoinNode;
}

bindings::export!(Component with_types_in bindings);

