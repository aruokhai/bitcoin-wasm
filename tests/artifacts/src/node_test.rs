use bindings::component::node::types::{ClientNode, NodeConfig, SocketAddress,  BitcoinNetwork};

use crate::bindings;


pub fn test_node(){
    let ip_config = SocketAddress{ ip: "127.0.0.1".to_string(), port: 18744 };
    let network_config = BitcoinNetwork::Regtest;
    let wallet_address = "bcrt1qlhwg8036lga3c2t4pmmc6wf49f8t0m5gshjzpj".to_string();
    let wallet_filter = "0014fddc83be3afa3b1c29750ef78d39352a4eb7ee88".to_string();
    let genesis_blockhash = "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206".to_string();
    let node = ClientNode::new(&NodeConfig{ socket_address: ip_config, network: network_config, wallet_address, wallet_filter, genesis_blockhash}, None);
    let balance = node.get_balance().unwrap();
    assert_eq!(balance, 10_0000_0000);


}