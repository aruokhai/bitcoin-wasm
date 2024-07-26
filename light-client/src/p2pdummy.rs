use std::{collections::HashMap, sync::atomic::AtomicUsize};

use wasi::{random::random, sockets::tcp::{InputStream, IpSocketAddress, OutputStream, TcpSocket}};
use bitcoin::{
    consensus::{encode, serialize, Decodable}, p2p::{message::{CommandString, NetworkMessage, RawNetworkMessage}, message_network::VersionMessage, Address}, Network
};

use crate::tcpsocket::WasiTcpSocket;



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
      pub fn send(&self, message: String){
            todo!()
      }

      pub async fn receive(&self, _message: String){
        todo!()
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
    peer: Peer,
    socket: WasiTcpSocket, 
}
pub trait  P2PControl {
    fn connect_peer(&mut self, address: IpSocketAddress) -> bool;
    fn disconnect_peer(&self) -> bool;
}

impl P2PControl for P2P {
    fn connect_peer(&mut self, remote_address: IpSocketAddress) -> bool {
        let connect_res = self.socket.blocking_connect(remote_address);
        match connect_res {
            Ok((input_stream, output_stream)) => {
                self.peer = Peer::new(input_stream, output_stream);
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

