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
    let ip_config = node::CustomIPV4SocketAddress{ ip: (127,0,0,1), port: 19444 };
    let network_config = node::WasiBitcoinNetwork::Regtest;


    // let node = node::Node::new(NodeConfig{ socket_address: ip_config, network: network_config});
   // println!("{:?}", response.status)
   let block_hash = Hash256::decode("7c8077cb3145dd6c3f46b00882b1aa585f0cc78eb2070510864aac3e0461f379").unwrap();
   let block_filter = b"0411b2d143324c5369b8b600".to_vec();
   let mut new_cursor = Cursor::new(block_filter);
   let script = b"0014c251c8b2840c62e2ce6399885a8611a25158fb52";


   let mut filter = util::block_filter::BlockFilterReader::new(&block_hash);
   filter.add_query_pattern(script);
   let result = filter.match_any(&mut new_cursor).unwrap();
   println!("result: {}", result);

}


