use bindings::component::node::types::{Node, Error, NodeConfig, SocketAddress,  BitcoinNetwork};

use crate::bindings;


pub fn test_node(){
    let ip_config = SocketAddress{ ip: "127.0.0.1".to_string(), port: 19446 };
    let network_config = BitcoinNetwork::Regtest;
    let wallet_address = "bcrt1qwt0qg7ujmhe3mllr94x3sef07lfa27u97ymdex".to_string();
    let wallet_filter = "001472de047b92ddf31dffe32d4d18652ff7d3d57b85".to_string();
    let genesis_blockhash = "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206".to_string();
    let node = Node::new(&NodeConfig{ socket_address: ip_config, network: network_config, wallet_address, wallet_filter, genesis_blockhash}, None);
    let balance = node.get_balance().unwrap();
    assert_eq!(balance, 5_0000_0000);


}