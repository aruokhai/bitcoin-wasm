use std::cell::RefCell;
use std::{hash::Hash, iter::zip, vec};

use bitcoin::{
    block, network as bitcoin_network,
};
use bindings::exports::component::node::types::{BitcoinNetwork as WasiBitcoinNetwork, NodeConfig as WasiNodeConfig};
use bindings::component::store::types::{Store,KeyValuePair, Error as StoreError };

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

#[derive(Debug)]
pub enum NodeError {
    NetworkError,
    FetchCompactFilter(util::Error),
    FetchCompactFilterHeader(util::Error),
    FetchBlock(util::Error),
    FetchTransaction(util::Error),
    FetchHeader(util::Error),
    StoreError(StoreError),
    SerializationError,
}


pub struct Node {
    p2p: P2P,
    last_block_hash: Hash256,
    last_block_height: u64,
    balance_sats: i64,
    store: RefCell<Store>,
}


impl Node {

    pub fn new(node_config: NodeConfig, mut store: RefCell<Store>) -> Self {
        let mut p2p = P2P::new();
        p2p.connect_peer(node_config.socket_address, node_config.network) .expect("Failed to connect to peer");

        let mut last_block_height= 0;
        let mut balance_sats =  0;
        let mut last_block_hash = node_config.genesis_blockhash;

        if let Ok(stored_last_block_hash) = store.get_mut().search(&"last_block_hash".to_string()) {
                last_block_hash = Hash256::decode(stored_last_block_hash.value.as_str()).expect("Cant decode block hash");
        }

        if let Ok(stored_balance_sat) = store.get_mut().search(&"balance_sats".to_string()) {
            balance_sats = stored_balance_sat.value.parse::<i64>().expect("Cant parse Balance");
        }

        if let Ok(stored_last_block_height) = store.get_mut().search(&"last_block_height".to_string()) {
            last_block_height = stored_last_block_height.value.parse::<u64>().expect("Cant parse BLock height Value");
        }

        Self { p2p, last_block_hash, last_block_height, balance_sats, store}

    }

    pub fn add_filter(& mut self, filter: String) -> Result<(), NodeError> {

        let mut stored_filter_scripts = self.get_filters()?;

        stored_filter_scripts.push(filter.into_bytes());

        let string_data: Vec<String> = stored_filter_scripts.iter()
        .map(|vec| String::from_utf8(vec.clone()).unwrap_or_else(|_| String::from("Invalid UTF-8")))
        .collect();

        // Serialize the Vec<String> to a JSON string
        let json_string = serde_json::to_string(&string_data).map_err(|_| NodeError::SerializationError)?;

        self.store.get_mut().insert(&KeyValuePair { key: "filter_scripts".to_string(), value: json_string}).map_err(|err| NodeError::StoreError(err))?;

        Ok(())
    }

    fn get_filters(& mut self) -> Result<Vec<Vec<u8>>, NodeError> {

        let filter_scripts = if let Ok(stored_filter_scripts) = self.store.get_mut().search(&"filter_scripts".to_string()) {
            let string_vec: Vec<String> = serde_json::from_str(&stored_filter_scripts.value).map_err(|_| NodeError::SerializationError)?;

            // Convert Vec<String> to Vec<Vec<u8>>
            let byte_vecs: Vec<Vec<u8>> = string_vec
                .iter()
                .map(|s| s.as_bytes().to_vec()) // Convert each String to Vec<u8>
                .collect(); // Collect results into Vec<Vec<u8>>
            
            byte_vecs
        } else { 
            vec![]
        };

        return Ok(filter_scripts)

    }

    
    fn get_and_verify_compact_filters(& mut self, start_height: u32, last_block_hash: Hash256) -> Result<Vec<CompactFilter>, NodeError> {
        let filter_header = self.p2p.get_compact_filter_headers(start_height, last_block_hash).map_err(|err| NodeError::FetchCompactFilterHeader(err))?;
        let filters = self.p2p.get_compact_filters(start_height, last_block_hash).map_err(|err| NodeError::FetchCompactFilter(err))?;

        
        for (filter_hash, compact_filter) in zip(filter_header.filter_hashes, filters.clone()) {
            let computed_hash = sha256d(&compact_filter.filter_bytes);
            assert_eq!(computed_hash, filter_hash, "Hash doesn't match for filter");
            println!("assertion 2 correct");
        }
        println!("all assertions  correct");
        return Ok(filters);
    }

    pub fn get_balance(& mut self) -> Result<i64, NodeError> {

        self.sync_balance()?;
        return Ok(self.balance_sats)

    }


    pub fn sync_balance(& mut self) -> Result<(),NodeError> {
        self.p2p.keep_alive().map_err(|_| NodeError::NetworkError)?;
        
        let mut last_block_hash = self.last_block_hash;
        let mut last_block_height = self.last_block_height;
        let mut balance_sats = self.balance_sats;
        let mut is_sync = true;
        const MAX_HEADER_LEN: usize = 2000;
 

        while is_sync {

            let block_headers = self.p2p.fetch_headers(last_block_hash)
            .map_err(|err| NodeError::FetchHeader(err))?;
            
            if block_headers.len() == 0 {
                return Ok(());
            }

            last_block_hash = block_headers.last()
                .expect("No block headers found")
                .hash();
    
            last_block_height = last_block_height+ block_headers.len() as u64;
            let filter_indexer = 500;
    
            
            let mut block_headers_counter = 0;
            let mut current_block_num = self.last_block_height + 1 ;
    
            while current_block_num <  last_block_height {
                
                let next_block_num = current_block_num + filter_indexer; 
                let last_know_block_hash = if next_block_num > last_block_height {
                    last_block_hash
                } else {
                    block_headers_counter += filter_indexer;
                    block_headers[block_headers_counter as usize -1].hash()
                };
    
                let block_filters = self.get_and_verify_compact_filters(current_block_num as u32,last_know_block_hash)?;
                if block_filters.is_empty() {
                    continue;
                }
                
                balance_sats += self.calculate_balance(block_filters)?;
                current_block_num = next_block_num;
            }
            if block_headers.len() < MAX_HEADER_LEN {
                is_sync = false;
            }
        }  

        self.balance_sats = balance_sats;
        self.last_block_height = last_block_height;
        self.last_block_hash = last_block_hash;

        self.store.get_mut().insert(&KeyValuePair { key: "balance_sats".to_string(), value: balance_sats.to_string()}).map_err(|err| NodeError::StoreError(err))?;
        self.store.get_mut().insert(&KeyValuePair { key: "last_block_height".to_string(), value: last_block_height.to_string()}).map_err(|err| NodeError::StoreError(err))?;
        self.store.get_mut().insert(&KeyValuePair { key: "last_block_hash".to_string(), value: last_block_hash.encode()}).map_err(|err| NodeError::StoreError(err))?;

        
        Ok(())
        
    }


    fn calculate_balance(&mut self, filters: Vec<CompactFilter>) -> Result<i64, NodeError> {

        let mut amount_sats = 0;

        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query = self.get_filters().unwrap().clone().into_iter();
            let result = filter_algo.match_any(&filter.block_hash, filter_query).unwrap();
            println!("{}", result);
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
        }).collect();


        let block_inv: Vec<_> = blockhash_present.into_iter().map(|hash| {
            InvVect{ obj_type: 2, hash }
        }).collect();

        let blocks = self.p2p.get_block(Inv{ objects: block_inv}).map_err(|err| NodeError::FetchBlock(err))?;


        for block in blocks {
             for txn in block.txns {
                 for output in txn.outputs {
                     if let Ok(filters) = self.get_filters(){
                        if filters.contains(&output.lock_script) {
                            amount_sats += output.satoshis;
                        }
                     }
                 }

                // TODO: Fix not working
                //  for input in txn.inputs {
                //     println!("this is prevout hash {:?}", input.prev_output.hash);
                //     if input.prev_output.hash == NO_HASH_STOP {
                //         continue;
                //     }
                    
                //     let transaction_inv = Inv{ objects: vec![InvVect{ obj_type: 0x40000001, hash: input.prev_output.hash }]};
                //     let prev = self.p2p.get_transaction(transaction_inv).map_err(|err| NodeError::FetchTransaction(err))?;
                //     let prev_out = &prev[0].outputs[input.prev_output.index as usize];
                //     if self.filter_scripts.contains(&prev_out.lock_script) {
                //         amount_sats -= prev_out.satoshis;
                //     }
                // }
             }
        }
        return  Ok(amount_sats);
    }

 
}
