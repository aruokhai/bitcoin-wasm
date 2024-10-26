#[allow(warnings)]
mod bindings;
use std::{cell::RefCell};

use node::Node;
use bindings::exports::component::node::types::{Guest,Error, GuestClientNode, NodeConfig};
use bindings::component::store::types::{Store };



mod node;
mod p2p;
mod tcpsocket;
mod util;
mod messages;

struct Component;

struct BitcoinNode {
    inner: RefCell<Node>,
}

struct KeyValuePair {
    key: String,
    value: String,
}


impl GuestClientNode for BitcoinNode {
    fn get_balance(&self) -> Result<i64, Error> {
        return  self.inner.borrow_mut().get_balance().map_err(|_| Error::NetworkError);
    }

    // fn new(config: NodeConfig, tbdx_config: Option<TbdexConfig>) -> Self {
    //     let tbdex = if let Some(config) = tbdx_config {
    //         let new_tbdex_client = Client::new(&config.pfi_uri, &config.vc_url, &config.acct_number);
    //         Some(RefCell::new(new_tbdex_client))
    //     } else {
    //         None
    //     };
    //     Self{ inner:  Node::new(config.into()).into(), tbdex}
    // }

    fn add_filter(&self, filter: String) -> Result<(), Error> {
        return  self.inner.borrow_mut().add_filter(filter).map_err(|_| Error::NetworkError);
    }

    fn new(config: NodeConfig) -> Self {
        Self{ inner:  Node::new(config.into(), Store::new().into()).into()}
    }
}

impl Guest for Component {
    
    type ClientNode  = BitcoinNode;
   
}

bindings::export!(Component with_types_in bindings);

