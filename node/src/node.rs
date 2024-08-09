use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network as bitcoin_network, p2p::{message::{CommandString, NetworkMessage, RawNetworkMessage}, message_network::VersionMessage}, Address, Network
};
use crate::p2p::{P2PControl, P2P};




struct Node {
    p2p: P2P,

}

pub struct CustomIPV4SocketAddress {
    pub ip: (u8,u8,u8,u8),
    pub port: u16
}

enum WasiBitcoinNetwork {
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

struct NodeConfig {
    socket_address: CustomIPV4SocketAddress,
    network: WasiBitcoinNetwork
}


impl Node {

    fn new(node_config: NodeConfig) -> Self {
        let mut p2p = P2P::new();
        let result = p2p.connect_peer(node_config.socket_address, node_config.network);
        if result == false {
            panic!("cant connect to peer");
        }

        return  Node { p2p };
    }

    
}
