use std::cell::RefCell;
use std::sync::Arc;
use std::{hash::Hash, iter::zip, vec};

use bitcoin::{
    block, network as bitcoin_network,
};
use bindings::exports::component::node::types::{BitcoinNetwork as WasiBitcoinNetwork,NodeConfig as WasiNodeConfig };
use bindings::component::kv::types::{Kvstore, Error as StoreError };

use crate::chain::CompactChain;
use crate::db::KeyValueDb;
use crate::util::Error;
use crate::{bindings, messages::{block::Block, compact_filter::{self, CompactFilter}, filter_locator::NO_HASH_STOP, headers, BlockHeader, Inv, InvVect}, p2p::{P2PControl, P2P}, util::{self, sha256d, Hash256}};



pub struct CustomIPV4SocketAddress {
    pub ip: (u8,u8,u8,u8),
    pub port: u16
}



impl From<WasiBitcoinNetwork> for bitcoin_network::Network {
    fn from(val: WasiBitcoinNetwork) -> Self {
        match val {
            WasiBitcoinNetwork::Mainnet => bitcoin_network::Network::Bitcoin,
            WasiBitcoinNetwork::Testnet => bitcoin_network::Network::Testnet,
            WasiBitcoinNetwork::Regtest => bitcoin_network::Network::Regtest,
        }
    }
}




impl From<WasiNodeConfig> for NodeConfig {
    fn from(val: WasiNodeConfig) -> Self {
        let WasiNodeConfig { network, socket_address, genesis_blockhash, wallet_address } = val;

        // Convert the network type
        let network: bitcoin_network::Network = network.into();

        // Parse the socket address IP
        let ip_segments: Vec<u8> = socket_address.ip
            .split('.')
            .filter_map(|segment| u8::from_str_radix(segment, 10).ok())
            .collect();

        if ip_segments.len() != 4 {
            panic!("Invalid IP address: {}", socket_address.ip);
        }

        // Construct the CustomIPV4SocketAddress
        let socket_address = CustomIPV4SocketAddress {
            ip: (ip_segments[0], ip_segments[1], ip_segments[2], ip_segments[3]),
            port: socket_address.port,
        };

        // Decode the  genesis blockhash with error handling
        let genesis_blockhash = Hash256::decode(&genesis_blockhash).expect("Failed to decode genesis blockhash");

        // Construct and return the NodeConfig
        NodeConfig {
            wallet_address,
            network,
            socket_address,
            genesis_blockhash,
        }
    }
}



pub struct NodeConfig {
    pub socket_address: CustomIPV4SocketAddress,
    pub network: bitcoin_network::Network,
    pub wallet_address: String,
    pub genesis_blockhash: Hash256,
}



pub struct Node {
    chain: CompactChain,
}


impl Node {

    pub fn new(node_config: NodeConfig, store: RefCell<Kvstore>) -> Self {
        let store = Arc::new(KeyValueDb::new(store)); 
        let chain = CompactChain::new(node_config.socket_address, node_config.network, store.clone());

        Self { chain }

    }

    pub fn balance(&mut self) -> Result<i64, Error> {
        self.chain.sync_state()?;
        let utxos = self.chain.get_utxos()?;

        return Ok(utxos.into_iter().fold(0, |acc, e| acc + e.tx_out.satoshis));  
    }

    pub fn add_filter(& mut self, filter: String) -> Result<(), Error> {
        let decoded_filter = hex::decode(filter).map_err(|e| Error::FromHexError(e))?;
        return self.chain.add_filter(decoded_filter);

    }


 
}
