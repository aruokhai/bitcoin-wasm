use std::{collections::HashMap, io::{Read, Write}, net::{Ipv4Addr, SocketAddrV4}, os::unix::net::SocketAddr, sync::atomic::AtomicUsize};

use wasi::{clocks::monotonic_clock, random::random, sockets::{instance_network, network::{self, Ipv4SocketAddress}, tcp::{InputStream, IpSocketAddress, OutputStream, TcpSocket}, tcp_create_socket::create_tcp_socket}};
use bitcoin::{
    consensus::{encode, serialize, Decodable, Encodable}, network, p2p::{message::{CommandString, NetworkMessage, RawNetworkMessage}, message_network::VersionMessage, Address}, Address, Network
};
use crate::tcpsocket::WasiTcpSocket;
use core::cmp;
use core::sync::atomic::Ordering;


pub struct PeerId(u64);


pub struct Peer {
    input_stream: InputStream,
    output_stream: OutputStream,
    peer_id: PeerId,
}

impl Peer {
      
      pub fn new(input_stream: InputStream, output_stream: OutputStream) -> Self {
         let peer_id = random::get_random_u64();
         Self { peer_id, input_stream, output_stream}
      }
      pub fn send(&self, message: RawNetworkMessage){
            let mut bytes = Vec::new();
            message.consensus_encode(&mut bytes).unwrap();
            self.output_stream.write_all(&bytes).unwrap();
            self.output_stream.blocking_flush().unwrap();

      }

      pub fn receive(&self) -> RawNetworkMessage{
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
    // serving others
    pub server: bool,
}

pub struct P2P {
    peer: Option<Peer>,
    socket: WasiTcpSocket, 
    bitcoin_config: BitcoinP2PConfig
}
pub trait  P2PControl {
    fn connect_peer(&mut self, address: CustomIPV4SocketAddress) -> bool;
    fn disconnect_peer(&self) -> bool;
}

impl P2P {

    pub fn new() -> Self {
        let raw_socket = create_tcp_socket(network::IpAddressFamily::Ipv4).unwrap();
        let network =  instance_network::instance_network();
        let wasi_socket = WasiTcpSocket::new(raw_socket, network);
        return P2P{ socket: wasi_socket, peer: None};
    }

    fn version (&self, remote_addrees: Address) -> NetworkMessage {
        // now in unix time
        let timestamp =  monotonic_clock::now().checked_div(1000).unwrap() as i64; 

        let services = 0;
        // build message
        NetworkMessage::Version(VersionMessage {
            version:  self.max_protocol_version,
            services,
            timestamp,
            receiver: remote_addrees.clone(),
            // sender is only dummy
            sender: remote_addrees,
            nonce: self.nonce,
            user_agent: self.bitcoin_config.user_agent.clone(),
            start_height: self.bitcoin_config.height.load(Ordering::Relaxed) as i32,
            relay: false,
        })
    }

    pub fn connect_acknowlege(&self, remote_adddress: Address) {
        let version_message = self.version(remote_adddress);
        self.peer.unwrap().send(version_message);
        let res = self.peer.unwrap().receive();
        if let NetworkMessage::Version(version) = res {
            let res = self.peer.unwrap().receive();
            if let NetworkMessage::Verack = res {
                let message = RawNetworkMessage::new(self.bitcoin_config.network.magic(), NetworkMessage::Verack);
                self.peer.unwrap().send(message);
            }
            panic!("cant get acknowledge version")
        }
        panic!("cant get version")

    }
}

struct CustomIPV4SocketAddress {
    ip: (u8,u8,u8,u8),
    port: u16
}

impl P2PControl for P2P {
    fn connect_peer(&mut self, remote_address: CustomIPV4SocketAddress) -> bool {
        // self.socket.blocking_bind(remote_address).unwrap();
        let wasi_socket_address = IpSocketAddress::Ipv4(Ipv4SocketAddress{ port: remote_address.port, address: remote_address.ip });
        let connect_res = self.socket.blocking_connect(remote_address);
        match connect_res {
            Ok((input_stream, output_stream)) => {
                self.peer = Some(Peer::new(input_stream, output_stream));
                let (a, b,c, d) = remote_address.ip;
                let bitcoin_socket_address = std::net::SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), remote_address.port))
                let bitcoin_remote_address = Address::new(&bitcoin_socket_address, 1);
                self.connect_acknowlege(bitcoin_remote_address);
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

