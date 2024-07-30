use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};

use crate::p2pdummy::{P2PControl, P2P};



struct Node {
    p2p: P2P,

}


impl Node {

    fn new(address: IpSocketAddress) -> Self {
        let mut p2p = P2P::new();
        let result = p2p.connect_peer(address);
        if result == false {
            panic!("cant connect to peer");
        }

        return  Node { p2p };
    }

    
}
