use std::{io::{Read, Write}, net::{Ipv4Addr, SocketAddrV4}, sync::atomic::AtomicUsize};

use wasi::{clocks::wall_clock, random::random, sockets::{instance_network, network::{self, Ipv4SocketAddress}, tcp::{InputStream, IpSocketAddress, OutputStream}, tcp_create_socket::create_tcp_socket}};
use bitcoin::{
    consensus::{encode, Decodable, Encodable}, network as bitcoin_network, Network
};
use crate::{node::CustomIPV4SocketAddress, tcpsocket::WasiTcpSocket};
use core::sync::atomic::Ordering;


pub struct PeerId(u64);
const MAX_PROTOCOL_VERSION: u32 = 70001;
const USER_AGENT: &'static str = concat!("/Murmel:", env!("CARGO_PKG_VERSION"), '/');

pub struct Peer {
    input_stream: InputStream,
    output_stream: OutputStream,
    peer_id: PeerId,
    remote_address: Address,
    bitcoin_config: BitcoinP2PConfig,
}

impl Peer {
      
      pub fn new(network: bitcoin_network::Network, input_stream: InputStream, output_stream: OutputStream, remote_address: Address) -> Self {
         let peer_id = random::get_random_u64();
         let bitcoin_config = BitcoinP2PConfig {
            network,
            nonce: random::get_random_u64(),
            max_protocol_version: MAX_PROTOCOL_VERSION,
            user_agent: USER_AGENT.to_owned(),
            height: AtomicUsize::new(0),
        };
         let peer =  Self { peer_id, input_stream, output_stream, remote_address, bitcoin_config};
         peer.handshake();
         return peer;
      }

      fn version (&self) -> NetworkMessage {
        // now in unix time
        let timestamp =  wall_clock::now().seconds;

        let services = 0;
        // build message
        NetworkMessage::Version(VersionMessage {
            version:  self.bitcoin_config.max_protocol_version,
            services: services.into(),
            timestamp: timestamp as i64,
            receiver: self.remote_address.clone(),
            // sender is only dummy
            sender: self.remote_address,
            nonce: self.bitcoin_config.nonce,
            user_agent: self.bitcoin_config.user_agent.clone(),
            start_height: self.bitcoin_config.height.load(Ordering::Relaxed) as i32,
            relay: false,
        })

    }

      fn handshake(&self) {
        let version_message = self.version();
        self.send(version_message);
        let res = self.receive();
        if let NetworkMessage::Version(version) = res {
            let res = self.receive();
            if let NetworkMessage::Verack = res {
                let message = RawNetworkMessage::new(self.bitcoin_config.network.magic(), NetworkMessage::Verack);
                self.send(message);
                let nonce = wall_clock::now().seconds;
                // Write a ping message because this seems to help with connection weirdness
                // https://bitcoin.stackexchange.com/questions/49487/getaddr-not-returning-connected-node-addresses
                let ping_message = NetworkMessage::Ping(nonce);
                self.send(ping_message);
            }
            panic!("cant get acknowledge version");
        }
        panic!("cant get version")
      }
    
        fn send(&mut self, message: RawNetworkMessage) {
            let mut bytes = Vec::new();
            message.consensus_encode(&mut bytes).unwrap();
            self.output_stream.write_all(&bytes).unwrap();
            self.output_stream.blocking_flush().unwrap();
      }

      pub 

       fn receive(&self) -> RawNetworkMessage{
        let mut new_vec = Vec::new();
        let bytes = self.input_stream.read_to_end(&mut new_vec).unwrap();
        let decoded_message: Result<RawNetworkMessage, encode::Error> =
            Decodable::consensus_decode::<RawNetworkMessage>(&mut new_vec);
        let message = decoded_message.unwrap();
        return  message;
        
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

    
    

}




impl P2PControl for P2P {
    fn connect_peer(&mut self, remote_address: CustomIPV4SocketAddress, network: bitcoin_network::Network) -> bool {
        let wasi_socket_address = IpSocketAddress::Ipv4(Ipv4SocketAddress{ port: remote_address.port, address: remote_address.ip });
        let connect_res = self.socket.blocking_connect(wasi_socket_address);
        match connect_res {
            Ok((input_stream, output_stream)) => {
                let (a, b,c, d) = remote_address.ip;
                let socket_address: std::net::SocketAddr = std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), remote_address.port));
                let remote_address =  Address::new(&socket_address, 1);
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

