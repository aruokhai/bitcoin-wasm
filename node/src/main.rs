#[allow(warnings)]
mod bindings;
mod node;
mod p2p;
mod tcpsocket;
mod util;
mod messages;

use std::{collections::HashMap, io::Cursor};

use node::NodeConfig;
use util::Hash256;
fn main() {
    println!("Hello, world!");
    let ip_config = node::CustomIPV4SocketAddress{ ip: (127,0,0,1), port: 19446 };
    let network_config = node::WasiBitcoinNetwork::Regtest;


    let node = node::Node::new(NodeConfig{ socket_address: ip_config, network: network_config});
    //println!("{:?}", response.status)
//    let block_hash = Hash256::decode("7c8077cb3145dd6c3f46b00882b1aa585f0cc78eb2070510864aac3e0461f379").unwrap();
//    let block_filter = hex::decode("0411b2d143324c5369b8b600").unwrap();
//    let  query =vec![hex::decode("0014c251c8b2840c62e2ce6399885a8611a25158fb52").unwrap()].into_iter();


//    let filter = util::block_filter::BlockFilter::new(&block_filter);
//    let result = filter.match_any(&block_hash, query).unwrap();
//    println!("result: {}", result);

}


