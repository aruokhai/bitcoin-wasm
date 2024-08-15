use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network as bitcoin_network, Network
};
use crate::{p2p::{P2PControl, P2P}, util::Hash256};




pub struct Node {
    p2p: P2P,

}

pub struct CustomIPV4SocketAddress {
    pub ip: (u8,u8,u8,u8),
    pub port: u16
}

pub enum WasiBitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

impl Into<bitcoin_network::Network> for WasiBitcoinNetwork {
    fn into(self) -> bitcoin_network::Network {
        match self {
            WasiBitcoinNetwork::Mainnet => bitcoin_network::Network::Bitcoin,
            WasiBitcoinNetwork::Testnet => bitcoin_network::Network::Testnet,
            WasiBitcoinNetwork::Regtest => bitcoin_network::Network::Regtest,
        }
    }
} 

pub struct NodeConfig {
    pub socket_address: CustomIPV4SocketAddress,
    pub network: WasiBitcoinNetwork
}


impl Node {

    pub fn new(node_config: NodeConfig) -> Self {
        let mut p2p = P2P::new();
        let result = p2p.connect_peer(node_config.socket_address, node_config.network.into());
        if result == false {
            panic!("cant connect to peer");
        }
        let mut reversed_string = String::new();

        let hash = "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206";
        // Iterate over the characters of the input string in reverse order
        // for c in hash.chars().rev() {
        //     reversed_string.push(c); // Append each character to the reversed string
        // }
        
        let last_known_blockhash  = Hash256::decode(hash).unwrap();
       p2p.sync_peer(last_known_blockhash);
        return  Node { p2p };
    }

    
}
