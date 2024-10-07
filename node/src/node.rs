use std::vec;

use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network as bitcoin_network, FilterHeader, Network
};
use bindings::exports::component::node::types::{BitcoinNetwork as WasiBitcoinNetwork, NodeConfig as WasiNodeConfig};

use crate::{bindings, messages::{block::Block, compact_filter::CompactFilter, filter_locator::NO_HASH_STOP, BlockHeader, Inv, InvVect}, p2p::{P2PControl, P2P}, util::{self, Hash256}, wallet::Wallet};



pub struct CustomIPV4SocketAddress {
    pub ip: (u8,u8,u8,u8),
    pub port: u16
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

impl Into<NodeConfig> for WasiNodeConfig {
    fn into(self) -> NodeConfig {
        let WasiNodeConfig { network, socket_address, genesis_blockhash, wallet_filter, wallet_address  } = self;
        let network: bitcoin_network::Network = network.into();
        
        let binding = socket_address.ip.clone();
        let ip_s: Vec<_> = binding.split(".").collect();
        let socket_address = CustomIPV4SocketAddress{ 
            ip: (u8::from_str_radix(ip_s[0], 10).unwrap()
                ,u8::from_str_radix(ip_s[1],10).unwrap()
                ,u8::from_str_radix(ip_s[2],10).unwrap()
                ,u8::from_str_radix(ip_s[3],10).unwrap()),
            port: socket_address.port
        };
        let wallet_filter = hex::decode(wallet_filter).unwrap();
        let genesis_blockhash = Hash256::decode(&genesis_blockhash).unwrap();

        return NodeConfig { wallet_address, network,  socket_address, wallet_filter, genesis_blockhash };
    }
}

pub struct NodeConfig {
    pub socket_address: CustomIPV4SocketAddress,
    pub network: bitcoin_network::Network,
    pub wallet_address: String,
    pub wallet_filter: Vec<u8>,
    pub genesis_blockhash: Hash256,
}

pub enum NodeError {
    NetworkError
}


pub struct Node {
    p2p: P2P,
    last_filter_header: Hash256,
    filter_scripts: Vec<Vec<u8>>,
    last_block_hash: Hash256,
    last_block_num: u64,
    balance_sats: i64,
}


impl Node {

    pub fn new(node_config: NodeConfig) -> Self {
        let mut p2p = P2P::new();
        p2p.connect_peer(node_config.socket_address, node_config.network.into()).unwrap();

        return Self { p2p, last_filter_header: NO_HASH_STOP, filter_scripts: vec![], last_block_hash: node_config.genesis_blockhash, last_block_num: 0, balance_sats: 0};

    }


    fn get_block_filters(& mut self) -> Vec<CompactFilter> {
        let block_headers = p2p.fetch_headers(self.last_block_hash).unwrap();
        let last_block_hash = block_headers.last()
            .expect("No block headers found")
            .hash();
        // TODO: verify block headers;

        let latest_block_num = self.last_block_num + block_headers.len();
        let mut filters: Vec<_> = Vec::new();
        
        let block_headers_counter = 0;
        let mut current_block_num = self.last_block_num ;

        while current_block_num <  latest_block_num {
            let next_block_num = current_block_num + 500; 
            let last_know_block_hash = if next_block_num > last_block_num {
                last_block_hash
            } else {
                block_headers_counter += 500;
                block_headers[block_headers_counter].hash()
            };

            let block_filters = self.p2p.get_compact_filters(current_block_num as u32,last_know_block_hash).unwrap();
            //TODO: Verify block filters;
            filters.extend(block_filters);
            current_block_num = next_block_num;
        }

        self.last_block_num = latest_block_num;
        self.last_block_hash = last_block_hash;
        return filters
    }
    
    pub fn get_balance(& mut self) -> std::result::Result<i64, NodeError> {
        self.p2p.keep_alive().map_err(|_| NodeError::NetworkError)?;

        let filters = self.get_block_filters();

        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query = self.filter_scripts.into_iter();
            let result = filter_algo.match_any(&filter.block_hash, filter_query).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
        }).collect();

        let block_inv: Vec<_> = blockhash_present.into_iter().map(|hash| {
            InvVect{ obj_type: 2, hash }
        }).collect();
        let blocks = p2p.get_block(Inv{ objects: block_inv}).unwrap();

        // TODO: account also for txn input
        let mut amount_sats = 0;
        for block in blocks {
             for txn in block.txns {
                 for output in txn.outputs {
                     if output.lock_script == node_config.wallet_filter.clone() {
                         amount_sats += output.satoshis;
                     }
                 }
             }
        }

        self.balance_sats = amount_sats;
        return Ok(amount_sats);
    }
 
}
