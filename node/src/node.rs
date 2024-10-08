use std::{iter::zip, vec};

use bitcoin::{
    block, network as bitcoin_network,
};
use bindings::exports::component::node::types::{BitcoinNetwork as WasiBitcoinNetwork, NodeConfig as WasiNodeConfig};

use crate::{bindings, messages::{block::Block, compact_filter::{self, CompactFilter}, filter_locator::NO_HASH_STOP, Inv, InvVect}, p2p::{P2PControl, P2P}, util::{self, sha256d, Hash256}};



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
        let WasiNodeConfig { network, socket_address, genesis_blockhash, wallet_filter, wallet_address  } = val;
        let network: bitcoin_network::Network = network.into();
        
        let binding = socket_address.ip.clone();
        let ip_s: Vec<_> = binding.split('.').collect();
        let socket_address = CustomIPV4SocketAddress{ 
            ip: (u8::from_str_radix(ip_s[0], 10).unwrap()
                ,u8::from_str_radix(ip_s[1],10).unwrap()
                ,u8::from_str_radix(ip_s[2],10).unwrap()
                ,u8::from_str_radix(ip_s[3],10).unwrap()),
            port: socket_address.port
        };
        let wallet_filter = hex::decode(wallet_filter).unwrap();
        let genesis_blockhash = Hash256::decode(&genesis_blockhash).unwrap();

        NodeConfig { wallet_address, network,  socket_address, wallet_filter, genesis_blockhash }
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
    NetworkError,
    FetchCompactFilter(util::Error),
    FetchCompactFilterHeader(util::Error),
    FetchBlock(util::Error),
    FetchTransaction(util::Error),
    FetchHeader(util::Error),
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
        p2p.connect_peer(node_config.socket_address, node_config.network).unwrap();

        Self { p2p, last_filter_header: NO_HASH_STOP, filter_scripts: vec![], last_block_hash: node_config.genesis_blockhash, last_block_num: 0, balance_sats: 0}

    }


    fn get_block_filters(& mut self) -> Result<Vec<CompactFilter>, NodeError> {
        let block_headers = self.p2p.fetch_headers(self.last_block_hash).map_err(|err| NodeError::FetchHeader(err))?;

        if block_headers.len() == 0 {
            return  Ok(Vec::new());
        }

        let last_block_hash = block_headers.last()
            .expect("No block headers found")
            .hash();

        let latest_block_num = self.last_block_num + block_headers.len() as u64;
        let mut filters: Vec<_> = Vec::new();
        
        let mut block_headers_counter = 0;
        let mut current_block_num = self.last_block_num ;

        while current_block_num <  latest_block_num {
            let next_block_num = current_block_num + 500; 
            let last_know_block_hash = if next_block_num > latest_block_num {
                last_block_hash
            } else {
                block_headers_counter += 500;
                block_headers[block_headers_counter].hash()
            };

            let (block_filters, latest_filter_header) = self.get_and_verify_compact_filters(current_block_num as u32,last_know_block_hash)?;
            filters.extend(block_filters);
            current_block_num = next_block_num;
            self.last_filter_header = latest_filter_header;
        }

        self.last_block_num = latest_block_num;
        self.last_block_hash = last_block_hash;

        Ok(filters)
    }
    
    fn get_and_verify_compact_filters(& mut self, start_height: u32, last_block_hash: Hash256) -> Result<(Vec<CompactFilter>, Hash256), NodeError> {
        let filterheader = self.p2p.get_compact_filter_headers(start_height, last_block_hash).map_err(|err| NodeError::FetchCompactFilterHeader(err))?;
        let filters = self.p2p.get_compact_filters(start_height, last_block_hash).map_err(|err| NodeError::FetchCompactFilter(err))?;

        let last_known_filter_header = self.last_filter_header;
        assert!(last_known_filter_header == filterheader.previous_filter_header, "last known filter header doesn't match");

        let last_filter_header = filterheader.clone().filter_hashes.into_iter().fold(last_known_filter_header,|acc, filter_hash| {
            return sha256d([acc.0, filter_hash.0].concat().as_slice())
        });
        
        for (filterhash, compact_filter) in zip(filterheader.filter_hashes, filters.clone()) {
            let computed_hash = sha256d(&compact_filter.filter_bytes);
            assert!(computed_hash == filterhash, "Hash Doesnt match");
        }

        return Ok((filters, last_filter_header));
    }

    pub fn get_balance(& mut self) -> Result<i64, NodeError> {
        self.p2p.keep_alive().map_err(|_| NodeError::NetworkError)?;

        let filters = self.get_block_filters()?;

        if filters.len() == 0 {
            return Ok(self.balance_sats);
        }

        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query = self.filter_scripts.clone().into_iter();
            let result = filter_algo.match_any(&filter.block_hash, filter_query).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
        }).collect();

        let block_inv: Vec<_> = blockhash_present.into_iter().map(|hash| {
            InvVect{ obj_type: 2, hash }
        }).collect();
        let blocks = self.p2p.get_block(Inv{ objects: block_inv}).map_err(|err| NodeError::FetchBlock(err))?;

        let amount_sats = self.calculate_balance(blocks)?;

        self.balance_sats = amount_sats;
        Ok(amount_sats)
    }


    fn calculate_balance(&mut self, blocks: Vec<Block>) -> Result<i64, NodeError> {
        let mut amount_sats = self.balance_sats;

        for block in blocks {
             for txn in block.txns {
                 for output in txn.outputs {
                     if self.filter_scripts.contains(&output.lock_script) {
                         amount_sats += output.satoshis;
                     }
                 }

                 for input in txn.inputs {
                    let transaction_inv = Inv{ objects: vec![InvVect{ obj_type: 1, hash: input.prev_output.hash }]};
                    let prev = self.p2p.get_transaction(transaction_inv).map_err(|err| NodeError::FetchTransaction(err))?;
                    let prev_out = &prev[0].outputs[input.prev_output.index as usize];
                    if self.filter_scripts.contains(&prev_out.lock_script) {
                        amount_sats -= prev_out.satoshis;
                    }
                }
             }
        }
        return  Ok(amount_sats);
    }

 
}
