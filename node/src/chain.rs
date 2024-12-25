use std::{iter::zip, sync::Arc};
use crate::{bindings, messages::{block_locator::NO_HASH_STOP, compact_filter::CompactFilter, tx_out::TxOut, Inv, InvVect}, util::{self, sha256d, Error}};

use bitcoin::network as bitcoin_network;
use bindings::component::kv::types::Error as StoreError ;
use serde::Serialize;

use crate::{db::KeyValueDb, node::CustomIPV4SocketAddress, p2p::{P2PControl, P2P}, util::Hash256};

pub struct CompactChain {
    p2p: P2P,
    db: Arc<KeyValueDb>,
    chain_state: ChainState,
}


#[derive(serde::Deserialize, Serialize)]
struct ChainState {
    last_block_hash: Hash256,
    last_block_height: u64,
    filters: Vec<Vec<u8>>,
    utxos: Vec<Utxo>
}

#[derive(serde::Deserialize, Serialize, Clone)]
pub struct Utxo  {
    pub tx_out: TxOut,
    hash: Hash256,
    index: usize,

}

const CHAIN_STATE_KEY: &str = "chain_state";
const MAX_HEADER_LEN: usize = 2000;
const FILTER_SIZE: usize = 500;


impl CompactChain {

    pub fn new(socket: CustomIPV4SocketAddress, network: bitcoin_network::Network, db: Arc<KeyValueDb>  ) -> Self {
        let mut p2p = P2P::new();
        p2p.connect_peer(socket, network).expect("Failed to connect to peer");

        match db.get(CHAIN_STATE_KEY.to_string()) {
             Ok(chain_state) =>  {
                let deserialized_chain_state: ChainState = bincode::deserialize(&chain_state).unwrap();
                Self{ p2p, db, chain_state: deserialized_chain_state }
             }
            Err(Error::DBError(StoreError::EntryNotFound)) => {
                let last_block_hash = Hash256::default();
                let last_block_height = 0;
                Self{ p2p, db, chain_state: ChainState{ last_block_hash, last_block_height, utxos: vec![], filters: vec![] } }
            },
            Err(_) => {
                panic!("Cannot get chainstate")
            }
        }
    }

    pub fn add_filter(& mut self, filter: Vec<u8>) -> Result<(), Error> {

        self.chain_state.filters.push(filter);
        let binary_chain_state: Vec<u8> = bincode::serialize(&self.chain_state).map_err(|e| Error::SerializationError(e.to_string()))?;
        self.db.insert(CHAIN_STATE_KEY.to_string(), binary_chain_state)?;

        Ok(())
    }

    pub fn get_utxos(& mut self) -> Result<Vec<Utxo>, Error> {
        return Ok(self.chain_state.utxos.clone());
    }

    fn get_and_verify_compact_filters(& mut self, start_height: u32, last_block_hash: Hash256) -> Result<Vec<CompactFilter>, Error> {
        let filter_header = self.p2p.get_compact_filter_headers(start_height, last_block_hash).map_err(|err| Error::FetchCompactFilterHeader(err.to_error_code()))?;
        let filters = self.p2p.get_compact_filters(start_height, last_block_hash).map_err(|err| Error::FetchCompactFilter(err.to_error_code()))?;

        
        for (filter_hash, compact_filter) in zip(filter_header.filter_hashes, filters.clone()) {
            let computed_hash = sha256d(&compact_filter.filter_bytes);
            if computed_hash != filter_hash {
                return Err(Error::FilterMatchEror)
            }
        }
        return Ok(filters);
    }

    fn fetch_and_save_utxos(&mut self, filters: Vec<CompactFilter>) -> Result<(), Error> {

        let blockhash_present: Vec<_> = filters.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let filter_query = &self.chain_state.filters;
            let result = filter_algo.match_any(&filter.block_hash, filter_query.clone().into_iter()).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
        }).collect();

        if blockhash_present.is_empty() {
            return Ok(());
        }

        

        let block_inv: Vec<_> = blockhash_present.into_iter().map(|hash| {
            InvVect{ obj_type: 2, hash }
        }).collect();

        let blocks = self.p2p.get_block(Inv{ objects: block_inv}).map_err(|err| Error::FetchBlock(err.to_error_code()))?;

        let mut new_utxos = self.chain_state.utxos.clone();
        for block in blocks {
             for txn in block.txns {
                 for (index, output) in txn.outputs.iter().enumerate() {
                    if self.chain_state.filters.contains(&output.lock_script) {
                        new_utxos.push(Utxo { tx_out: output.to_owned(), hash: txn.hash(), index });
                    }
                 }

                for input in txn.inputs {
                   if input.prev_output.hash == NO_HASH_STOP {
                       continue;
                   } 

                   //TODO: Ensure all inputs are included
                   for (index,utxo) in  new_utxos.clone().iter().enumerate() {
                        if utxo.hash == input.prev_output.hash && utxo.index as u32 == input.prev_output.index {
                            new_utxos.remove(index);
                        }       
                   } 
                }
             }
        }
        self.chain_state.utxos = new_utxos;
        let binary_chain_state: Vec<u8> = bincode::serialize(&self.chain_state).map_err(|e| Error::SerializationError(e.to_string()))?;
        self.db.insert(CHAIN_STATE_KEY.to_string(), binary_chain_state)?;

        Ok(())

    }

    pub fn sync_state(& mut self) -> Result<(),Error> {
        self.p2p.keep_alive().map_err(|_| Error::NetworkError)?;

        let mut is_sync = true;

        while is_sync {

            let fetched_block_headers = self.p2p.fetch_headers(self.chain_state.last_block_hash)
            .map_err(|err| Error::FetchHeader(err.to_error_code()))?;
            if fetched_block_headers.len() == 0 {
                return Ok(());
            }

            let last_block_hash = fetched_block_headers.last()
                .expect("No block headers found")
                .hash();

            // Calculate the range for the for loop
            let start_block = self.chain_state.last_block_height + 1;
            let end_block = self.chain_state.last_block_height + fetched_block_headers.len() as u64;
            // Generate ranges for block numbers and block headers counter
            let block_numbers = (start_block..end_block).step_by(FILTER_SIZE as usize);
            let block_headers_counters = (FILTER_SIZE..).step_by(FILTER_SIZE as usize);

            // Use zip to iterate over both ranges simultaneously
            for (current_block_num, block_headers_counter) in block_numbers.zip(block_headers_counters) {
                // Get the last known block hash
                let last_known_block_hash = fetched_block_headers
                    .get(block_headers_counter as usize - 1)
                    .map(|header| header.hash())
                    .unwrap_or(last_block_hash);
            
                // Fetch and verify compact filters for the current range
                let block_filters = self.get_and_verify_compact_filters(
                    current_block_num as u32,
                    last_known_block_hash,
                )?;

                if block_filters.is_empty() {
                    continue;
                }
                        
                // Fetch and save UTXOs for the verified block filters
                self.fetch_and_save_utxos(block_filters)?;
            }
    
            if fetched_block_headers.len() < MAX_HEADER_LEN {
                is_sync = false;
            }

            self.chain_state.last_block_height = end_block;
            self.chain_state.last_block_hash = last_block_hash;
        }  

        

        let binary_chain_state: Vec<u8> = bincode::serialize(&self.chain_state).map_err(|e| Error::SerializationError(e.to_string()))?;
        self.db.insert(CHAIN_STATE_KEY.to_string(), binary_chain_state)?;

        Ok(())
        
    }


    
}
