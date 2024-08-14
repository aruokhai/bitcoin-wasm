#[allow(warnings)]
mod bindings;
mod node;
mod p2p;
mod tcpsocket;
mod util;
mod messages;

use std::collections::HashMap;

use node::NodeConfig;
fn main() {
    println!("Hello, world!");
    let ip_config = node::CustomIPV4SocketAddress{ ip: (127,0,0,1), port: 19446 };
    let network_config = node::WasiBitcoinNetwork::Regtest;


    let node = node::Node::new(NodeConfig{ socket_address: ip_config, network: network_config});
   // println!("{:?}", response.status)

}


