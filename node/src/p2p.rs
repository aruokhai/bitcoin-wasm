use std::{io::{Read, Write}, net::{Ipv4Addr}, str::FromStr, sync::atomic::AtomicUsize};

use wasi::{clocks::{monotonic_clock, wall_clock}, random::random, sockets::{instance_network, network::{self, Ipv4SocketAddress}, tcp::{InputStream, IpSocketAddress, OutputStream}, tcp_create_socket::create_tcp_socket}};
use bitcoin::{
    network as bitcoin_network, Network
};
use crate::{messages::{self, block::Block, block_locator::{BlockLocator, NO_HASH_STOP }, commands::{self, PONG}, compact_filter::CompactFilter, compact_filter_header::CompactFilterHeader, filter_locator::FilterLocator, tx::Tx, BlockHeader, Inv, Message, NodeAddr, Version, PROTOCOL_VERSION}, util::Hash256};
use crate::node::CustomIPV4SocketAddress;
use crate::tcpsocket::WasiTcpSocket;
use core::sync::atomic::Ordering;
use crate::messages::Message::Ping;
use crate::util::{Error, Result};

const MAX_PROTOCOL_VERSION: u32 = 70015;
const USER_AGENT: &str = concat!("/BITCOINWASM:", env!("CARGO_PKG_VERSION"), '/');

pub struct Peer {
    input_stream: InputStream,
    output_stream: OutputStream,
    remote_address: NodeAddr,
    bitcoin_config: BitcoinP2PConfig,
}

impl Peer {
      
    pub fn new(network: bitcoin_network::Network, input_stream: InputStream, output_stream: OutputStream, remote_address: NodeAddr) -> Self {
      let bitcoin_config = BitcoinP2PConfig {
         network,
         nonce: random::get_random_u64(),
         max_protocol_version: MAX_PROTOCOL_VERSION,
         user_agent: USER_AGENT.to_owned(),
         height: AtomicUsize::new(0),
      };
      let mut peer =  Self { input_stream, output_stream, remote_address, bitcoin_config};
      peer.handshake().unwrap();
      peer
    }

    fn version (&self) -> Message {
      // now in unix time
      let timestamp =  wall_clock::now().seconds;
      let services = 0;
      // build message
      Message::Version(Version {
          version:  self.bitcoin_config.max_protocol_version,
          services: services as u64,
          timestamp: timestamp as i64,
          recv_addr: self.remote_address.clone(),
          // sender is only dummy
          tx_addr: self.remote_address.clone(),
          nonce: self.bitcoin_config.nonce,
          user_agent: self.bitcoin_config.user_agent.clone(),
          start_height: self.bitcoin_config.height.load(Ordering::Relaxed) as i32,
          relay: false,
      })

    }

      fn handshake(&mut self) -> Result<()> {
        let version_message = self.version();
        self.send(version_message)?;
        let res = self.receive(commands::VERSION)?;

        if let Message::Version(_) = res {
            let res = self.receive(commands::VERACK)?;

            if let Message::Verack = res {
                let message = Message::Verack;
                self.send(message)?;
                let nonce = random::get_random_u64();
                
                let ping_message = Ping(messages::ping::Ping { nonce });
                self.send(ping_message)?;
                self.receive(PONG)?;

                println!("handshake complete");

                return Ok(());
            }
        }
        Err(Error::WrongP2PMessage)
      }

      pub fn fetch_headers(& mut self, last_known_blockhash: Hash256) -> Result<Vec<BlockHeader>> {
            let block_locator = BlockLocator{ version: PROTOCOL_VERSION, block_locator_hashes: vec![last_known_blockhash], hash_stop:  NO_HASH_STOP};
            self.send(Message::GetHeaders(block_locator))?;

            if let Message::Headers(headers) =  self.receive(commands::HEADERS)?{
                return Ok(headers.inner)
            }
            return Err(Error::WrongP2PMessage);
      }

      pub fn fetch_compact_filters(& mut self, start_height: u32, hash_stop: Hash256 ) ->  Result<Vec<CompactFilter>> {
            let compact_locator = FilterLocator { filter_type: 0, start_height, hash_stop};
            let mut block_filters = Vec::new();
            self.send(Message::GetCFilters(compact_locator))?;

            loop {
                if let Message::CFilters(filters) =  self.receive(commands::CFILTERS)?{
                    block_filters.push(filters.clone());
                    if filters.block_hash == hash_stop {
                        return Ok(block_filters);
                    }
                    continue;
                }
                return Err(Error::WrongP2PMessage);
            }
      }

      pub fn fetch_compact_filter_headers(& mut self, start_height: u32, hash_stop: Hash256 ) ->  Result<CompactFilterHeader> {
        let compact_locator = FilterLocator { filter_type: 0, start_height, hash_stop};
        println!("initiated compact filter header");
        self.send(Message::GetCFHeaders(compact_locator))?;

        if let Message::CFHeaders(cfheades) =  self.receive(commands::CFHEADERS)? {
            return Ok(cfheades);
        }
        Err(Error::WrongP2PMessage)
  }

      pub fn keep_alive(& mut self) -> Result<()> {
            let nonce = random::get_random_u64();
            let ping_message = Ping(messages::ping::Ping { nonce });
            self.send(ping_message)?;

            match self.receive(PONG) {
                Ok(_) => {
                    println!("initialted already");

                    Ok(())
                },
                Err(_) => {
                    self.handshake()
                },
            }
      }

      pub fn fetch_blocks(& mut self, inv: Inv) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        let data_len = inv.objects.len();
        self.send(Message::GetData(inv))?;

        loop {
            if let Message::Block(block) =  self.receive(commands::BLOCK)?{
                blocks.push(block.clone());
                if blocks.len() == data_len {
                    return Ok(blocks);
                } 
                continue;
            }
            return Err(Error::WrongP2PMessage);
        }
    }

    pub fn fetch_transactions(& mut self, inv: Inv) -> Result<Vec<Tx>> {
        let mut transactions = Vec::new();
        let data_len = inv.objects.len();
        self.send(Message::GetData(inv))?;
        println!("data here");
        loop {
            match self.receive(commands::TX)? {
                Message::Tx(transaction) => {
                    println!("gotten txn_Data");
                    transactions.push(transaction.clone());
                    if transactions.len() == data_len {
                        return Ok(transactions);
                    } 
                    continue;
                },  
                Message::NotFound(inv) => {
                    println!("data not found {:?}", inv);
                    return Err(Error::WrongP2PMessage);
                },
                _ => {
                    return Err(Error::WrongP2PMessage);
                }
            }
        }
    }
    
        fn send(&mut self, message: Message) -> Result<()> {
            message.write(&mut self.output_stream, [0xfa, 0xbf, 0xb5, 0xda]).map_err(Error::IOError)?;
            self.output_stream.blocking_flush().map_err(Error::StreamingError)?;
            Ok(())
      }

       

    fn receive(& mut self, message_type: [u8; 12]) -> Result<Message>{
         let duration = monotonic_clock::now() + 1_000_000_000;
         while monotonic_clock::now() < duration {
             let decoded_message = Message::read(&mut self.input_stream);
             match decoded_message{
                 Ok(message) => {
                    if message.1.command == commands::NOTFOUND {
                        return Ok(message.0)
                    }
                     if message.1.command == message_type {
                         return Ok(message.0)
                     }
                 },
                 Err(Error::IOError(_)) => continue,
                 Err(err) => {
                     return Err(err)
                 }
             }   
         }
        Err(Error::Timeout)    
    }

}
pub struct BitcoinP2PConfig {
    pub network: Network,
    // This node's identifier on the network (random)
    pub nonce: u64,
    // height of the blockchain tree trunk
    pub height: AtomicUsize,
    // This node's human readable type identification
    pub user_agent: String,
    // this node's maximum protocol version
    pub max_protocol_version: u32,
}

pub struct P2P {
    peer: Option<Peer>,
    socket: WasiTcpSocket, 
}
pub trait  P2PControl {
    fn connect_peer(&mut self, address: CustomIPV4SocketAddress, network: bitcoin_network::Network) -> Result<()>;
}

impl P2PControl for P2P {
    fn connect_peer(&mut self, remote_address: CustomIPV4SocketAddress, network: bitcoin_network::Network) -> Result<()> {
        let wasi_socket_address = IpSocketAddress::Ipv4(Ipv4SocketAddress{ port: remote_address.port, address: remote_address.ip });
        let connect_res = self.socket.blocking_connect(wasi_socket_address);

        match connect_res {
            Ok((input_stream, output_stream)) => {
                let (a, b,c, d) = remote_address.ip;
                let socket_address = std::net::IpAddr::V4(Ipv4Addr::new(a, b, c, d));
                let remote_address = NodeAddr::new(socket_address, remote_address.port); 
                let peer = Peer::new(network, input_stream, output_stream, remote_address);
                self.peer = Some(peer);
                Ok(())
            },
            Err(e) => {
                Err(Error::TCPError(e))
            },
        }
    }
    

}

    impl P2P {

        pub fn new() -> Self {
            let raw_socket = create_tcp_socket(network::IpAddressFamily::Ipv4).expect("cant create socket");
            let network =  instance_network::instance_network();
            let wasi_socket = WasiTcpSocket::new(raw_socket, network);
            P2P{ socket: wasi_socket, peer: None}
        }

        pub fn fetch_headers(&mut self, last_known_blockhash: Hash256) -> Result<Vec<BlockHeader>> {
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .fetch_headers(last_known_blockhash)
        }
    
        pub fn get_compact_filters(&mut self, start_height: u32, hash_stop: Hash256) -> Result<Vec<CompactFilter>> { 
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .fetch_compact_filters(start_height, hash_stop)
        }

        pub fn get_compact_filter_headers(&mut self, start_height: u32, hash_stop: Hash256) -> Result<CompactFilterHeader> { 
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .fetch_compact_filter_headers(start_height, hash_stop)
        }
    
        pub fn get_block(&mut self, inv: Inv) -> Result<Vec<Block>> {
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .fetch_blocks(inv)
        }

        pub fn get_transaction(&mut self, inv: Inv) -> Result<Vec<Tx>> {
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .fetch_transactions(inv)
        }
    
        pub fn keep_alive(&mut self) -> Result<()> {
            self.peer
                .as_mut()
                .ok_or(Error::PeerNotFound)?
                .keep_alive()
        }
        
    }





