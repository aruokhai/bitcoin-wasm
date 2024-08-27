use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network as bitcoin_network, Network
};
use bindings::exports::component::node::types::{BitcoinNetwork as WasiBitcoinNetwork, NodeConfig as WasiNodeConfig};

use crate::{bindings, messages::{block::Block, BlockHeader, Inv, InvVect}, p2p::{P2PControl, P2P}, util::{self, Hash256}, wallet::Wallet};




pub struct Node {
    p2p: P2P,
    headers: Vec<BlockHeader>,
    last_block_hash: Hash256,
    last_block_num: u64,
    wallet: Wallet
}



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
        let wallet_address = hex::decode(wallet_address).unwrap();
        let wallet_filter = hex::decode(wallet_filter).unwrap();
        let genesis_blockhash = Hash256::decode(&genesis_blockhash).unwrap();

        return NodeConfig { wallet_address, network,  socket_address, wallet_filter, genesis_blockhash };
    }
}

pub struct NodeConfig {
    pub socket_address: CustomIPV4SocketAddress,
    pub network: bitcoin_network::Network,
    pub wallet_address: Vec<u8>,
    pub wallet_filter: Vec<u8>,
    pub genesis_blockhash: Hash256,
}

pub enum NodeError {
    NetworkError
}


impl Node {

    pub fn new(node_config: NodeConfig) -> Self {
        let mut p2p = P2P::new();
        p2p.connect_peer(node_config.socket_address, node_config.network.into()).unwrap();
        let block_headers = p2p.fetch_headers(node_config.genesis_blockhash).unwrap();
        let last_block_hash = block_headers.last().clone().unwrap().hash();
        let last_block_num = block_headers.len();
        let mut filters = Vec::new();
        let mut counter = 0 ;
        while counter <= last_block_num  {
            let next_counter = counter + 500;
            let last_know_block_hash = block_headers[counter].hash();
            let block_filters = p2p.get_compact_filters(counter as u32,last_know_block_hash).unwrap();
            filters.extend(block_filters);
            counter = next_counter;
        }
        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query =vec![node_config.wallet_filter.clone()].into_iter();
            let result = filter_algo.match_any(&filter.block_hash, filter_query).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
       }).collect();
       let inv_objects: Vec<_> = blockhash_present.into_iter().map(|hash| {
                InvVect{ obj_type: 2, hash }
           }).collect();
        let blocks = p2p.get_block(Inv{ objects: inv_objects}).unwrap();
        let mut amount_sats = 0;
        for block in blocks {
             for txn in block.txns {
                 for output in txn.outputs {
                     if output.lock_script == hex::decode("0014c251c8b2840c62e2ce6399885a8611a25158fb52").unwrap() {
                         amount_sats += output.satoshis;
                     }
                 }
             }
        }
        let wallet = Wallet { address: hex::decode(node_config.wallet_address).unwrap(), p2wkh_script: node_config.wallet_filter ,
            amount_sats };
        return  Node { p2p, headers: block_headers, last_block_hash, last_block_num : last_block_num as u64, wallet };   
    }
    
    pub fn get_balance(& mut self) -> std::result::Result<i64, NodeError> {
        self.p2p.ping_or_reconnect().map_err(|_| NodeError::NetworkError)?;
        let p2wkh_script = self.wallet.p2wkh_script.clone();
        let block_headers = self.p2p.fetch_headers(self.last_block_hash).unwrap();
        let mut old_last_block_num = self.last_block_num;
        self.last_block_num = old_last_block_num + block_headers.len() as u64;
        let mut filters = Vec::new();
        while old_last_block_num <= self.last_block_num  {
            let next_counter = old_last_block_num + 500;
            let last_know_block_hash = block_headers[old_last_block_num as usize].hash();
            let block_filters = self.p2p.get_compact_filters(old_last_block_num as u32,last_know_block_hash).unwrap();
            filters.extend(block_filters);
            old_last_block_num = next_counter;
        }
        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query =vec![p2wkh_script.clone()].into_iter();
            let result = filter_algo.match_any(&filter.block_hash, filter_query).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
       }).collect();
       let inv_objects: Vec<_> = blockhash_present.into_iter().map(|hash| {
                InvVect{ obj_type: 2, hash }
           }).collect();
        let blocks = self.p2p.get_block(Inv{ objects: inv_objects}).unwrap();
        let mut amount_sats = self.wallet.amount_sats;
        for block in blocks {
             for txn in block.txns {
                 for output in txn.outputs {
                     if output.lock_script == hex::decode("0014c251c8b2840c62e2ce6399885a8611a25158fb52").unwrap() {
                         amount_sats += output.satoshis;
                     }
                 }
             }
        }
        self.wallet.amount_sats = amount_sats;
        return Ok(amount_sats);
    }




    
}
