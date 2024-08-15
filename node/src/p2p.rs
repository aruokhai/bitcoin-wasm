use std::{io::{Cursor, Read, Write}, net::{Ipv4Addr, SocketAddrV4}, sync::atomic::AtomicUsize};

use wasi::{cli::command, clocks::{monotonic_clock, wall_clock}, io::poll::Pollable, random::random, sockets::{instance_network, network::{self, Ipv4SocketAddress}, tcp::{InputStream, IpSocketAddress, OutputStream}, tcp_create_socket::create_tcp_socket}};
use bitcoin::{
    block::Header, consensus::{encode, Decodable, Encodable}, network as bitcoin_network, Network
};
use crate::{messages::{self, block_locator::{self, BlockLocator, NO_HASH_STOP }, commands::{self, PING, PONG}, BlockHeader, Message, MessageHeader, NodeAddr, Version, PROTOCOL_VERSION}, util::Hash256};
use crate::node::CustomIPV4SocketAddress;
use crate::tcpsocket::WasiTcpSocket;
use core::sync::atomic::Ordering;
use crate::messages::Message::Ping;
use crate::util::{Error, Result, Serializable};


pub struct PeerId(u64);
const MAX_PROTOCOL_VERSION: u32 = 70015;
const USER_AGENT: &'static str = concat!("/Murmel:", env!("CARGO_PKG_VERSION"), '/');

pub struct Peer {
    input_stream: InputStream,
    output_stream: OutputStream,
    peer_id: u64,
    remote_address: NodeAddr,
    bitcoin_config: BitcoinP2PConfig,
}

pub enum MessageType {
    VERSION,
    Verack


}

impl Peer {
      
      pub fn new(network: bitcoin_network::Network, input_stream: InputStream, output_stream: OutputStream, remote_address: NodeAddr) -> Self {
         let peer_id = random::get_random_u64();
         let bitcoin_config = BitcoinP2PConfig {
            network,
            nonce: random::get_random_u64(),
            max_protocol_version: MAX_PROTOCOL_VERSION,
            user_agent: USER_AGENT.to_owned(),
            height: AtomicUsize::new(0),
        };
         let mut peer =  Self { peer_id, input_stream, output_stream, remote_address, bitcoin_config};
        peer.handshake();
         return peer;
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

      fn handshake(&mut self) {
        let version_message = self.version();
        self.send(version_message);
        let res = self.receive(commands::VERSION);
        if let Message::Version(version) = res {
            println!("{:?}", version);
            println!("version recieved");
            let res = self.receive(commands::VERACK);
            if let Message::Verack = res {
                println!("version acknowledge");
                let message = Message::Verack;
                self.send(message);
                let nonce = random::get_random_u64();
                // // Write a ping message because this seems to help with connection weirdness
                // // https://bitcoin.stackexchange.com/questions/49487/getaddr-not-returning-connected-node-addresses
                let ping_message = Ping(messages::ping::Ping { nonce: nonce });
                self.send(ping_message);
                self.receive(PONG);
                println!("pinged");
                
                return;
            }
            // self.handshake();
            panic!("cant get verack")
        }
        panic!("cant get version")
      }

      pub fn sync_headers(& mut self, last_known_blockhash: Hash256) -> Vec<BlockHeader> {
            let block_locator = BlockLocator{ version: PROTOCOL_VERSION, block_locator_hashes: vec![last_known_blockhash], hash_stop: NO_HASH_STOP };
            let mut block_headers = Vec::new();
            self.send(Message::GetHeaders(block_locator));
            println!("gptten here");
            loop {
                if let Message::Headers(headers) =  self.receive(commands::HEADERS){
                    println!("headers gooten");
                    block_headers.extend(headers.headers.clone());
                    if headers.headers.len() < 2000 {
                        return block_headers;
                    } 
                    let new_block_hash = headers.headers.last().clone().unwrap().to_owned().hash();
                    println!("{:?}", new_block_hash);
                    let new_block_locator = BlockLocator{ version: PROTOCOL_VERSION, block_locator_hashes: vec![new_block_hash], hash_stop: NO_HASH_STOP };
                    self.send(Message::GetHeaders(new_block_locator))

                }
                panic!("cant get headers");
            }

      }
    
        fn send(&mut self, message: Message) {
            message.write(&mut self.output_stream, [0xfa, 0xbf, 0xb5, 0xda]).unwrap();
            self.output_stream.blocking_flush().unwrap();
      }

       

       fn receive(& mut self, message_type: [u8; 12]) -> Message{
        let duration = monotonic_clock::now() + 10_000_000_000;
        while monotonic_clock::now() < duration {
            
            println!("trying");
            let decoded_message = Message::read(&mut self.input_stream);
            match decoded_message{
                Ok(message) => {
                    if message.1.command == commands::VERACK {
                        println!("Verack message gotten")
                    }
        
                    if message.1.command == message_type {
                        return message.0
                    }
                },
                Err(Error::IOError(_)) => continue,
                Err(_) => break
            }
            
            
            
        }
        panic!("cant get message");        
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
    fn connect_peer(&mut self, address: CustomIPV4SocketAddress, network: bitcoin_network::Network) -> bool;
    fn disconnect_peer(&self) -> bool;
}

impl P2P {

    pub fn new() -> Self {
        let raw_socket = create_tcp_socket(network::IpAddressFamily::Ipv4).unwrap();
        let network =  instance_network::instance_network();
        let wasi_socket = WasiTcpSocket::new(raw_socket, network);
        return P2P{ socket: wasi_socket, peer: None};
    }

    pub fn sync_peer(&mut self, last_known_blockhash: Hash256) {
        let headers = self.peer.as_mut().unwrap().sync_headers(last_known_blockhash);
        println!("This is your headers {:?}", headers)
    }
    
    

}




impl P2PControl for P2P {
    fn connect_peer(&mut self, remote_address: CustomIPV4SocketAddress, network: bitcoin_network::Network) -> bool {
        let wasi_socket_address = IpSocketAddress::Ipv4(Ipv4SocketAddress{ port: remote_address.port, address: remote_address.ip });
        let connect_res = self.socket.blocking_connect(wasi_socket_address);
        match connect_res {
            Ok((input_stream, output_stream)) => {
                let (a, b,c, d) = remote_address.ip;
                let socket_address = std::net::IpAddr::V4(Ipv4Addr::new(a, b, c, d));
                let remote_address = NodeAddr::new(socket_address, remote_address.port); 
                let peer = Peer::new(network, input_stream, output_stream, remote_address);
                self.peer = Some(peer);
                return  true;
            },
            Err(_) => {
                return false;
            },
        }
    }
    
    

    fn disconnect_peer(&self) -> bool {
        todo!()
    }
}

