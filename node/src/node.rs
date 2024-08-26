use wasi::sockets::{network::IpAddress, tcp::IpSocketAddress};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network as bitcoin_network, Network
};
use crate::{messages::{block::Block, BlockHeader, Inv, InvVect}, p2p::{P2PControl, P2P}, util::{self, Hash256}};




pub struct Node {
    p2p: P2P,
    headers: Vec<BlockHeader>

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

        let hash = "515f65013dbd143cfc76951cd8dedb432d1f14ae309a7a2d6667532042234a30";
        // Iterate over the characters of the input string in reverse order
        // for c in hash.chars().rev() {
        //     reversed_string.push(c); // Append each character to the reversed string
        // }
        
       let last_known_blockhash  = Hash256::decode(hash).unwrap();
       //let block_headers = p2p.sync_peer(last_known_blockhash);
       let block_filts = p2p.get_compact_filters(0,last_known_blockhash);
       let blockhash_present: Vec<_> = block_filts.into_iter().filter_map(|filter| {
            let filter_algo = util::block_filter::BlockFilter::new(&filter.filter_bytes);
            let  query =vec![hex::decode("0014c251c8b2840c62e2ce6399885a8611a25158fb52").unwrap()].into_iter();
            let result = filter_algo.match_any(&filter.block_hash, query).unwrap();
            match result {
                true => Some(filter.block_hash),
                false => None,
            }
       }).collect();

      
       let inv_objects: Vec<_> = blockhash_present.into_iter().map(|hash| {
            InvVect{ obj_type: 2, hash }
       }).collect();
       let mut blocks = p2p.get_block(Inv{ objects: inv_objects});
      

       let mut amount = 0;
       for block in blocks {
            for txn in block.txns {
                for output in txn.outputs {
                    if output.lock_script == hex::decode("0014c251c8b2840c62e2ce6399885a8611a25158fb52").unwrap() {
                        amount += output.satoshis;
                    }
                }
            }
       }
        
       println!("{:?}", amount);


        return  Node { p2p, headers: vec![] };
    }
    


    
}
