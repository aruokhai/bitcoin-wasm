use std::vec;

use crate::p2p::{Peer, P2P};

#[derive(Debug)]
pub struct CompactChain {
    peer: P2P,
}

impl CompactChain {

    fn new(node_config: NodeConfig) -> Self {
        let mut p2p = P2P::new();
        p2p.connect_peer(node_config.socket_address, node_config.network).expect("Failed to connect to peer");
        Self{ peer: p2p }
    }

    
}
