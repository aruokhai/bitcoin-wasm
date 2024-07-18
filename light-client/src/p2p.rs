/// The P2P network layer
/// 
/// 
/// 
/// 
use bitcoin::{
    consensus::{encode, serialize, Decodable}, p2p::{message::{CommandString, NetworkMessage, RawNetworkMessage}, message_network::VersionMessage, Address}
};
use mio::{Poll, Token, Waker};
use core::marker::PhantomData;
use std::{cmp::{min, Ordering}, collections::HashMap, io, sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, time::UNIX_EPOCH};
use tokio::{sync::{mpsc}};
use core::fmt;
use core::sync::atomic::AtomicUsize;
use wasi::{sockets::{network::{IpSocketAddress, Ipv4SocketAddress}, tcp::{InputStream, Network, OutputStream, Pollable, TcpSocket}, tcp_create_socket::create_tcp_socket, *}};

use crate::buffer::Buffer;


// use bitcoin::network:: {
//     address::Address,
//     constants::Network,
//     message::{NetworkMessage, RawNetworkMessage},
//     message_network::VersionMessage
// };

const IO_BUFFER_SIZE:usize = 1024*1024;
const EVENT_BUFFER_SIZE:usize = 1024;
const CONNECT_TIMEOUT_SECONDS: u64 = 5;
const BAN :u32 = 100;

/// do we serve blocks?
pub const SERVICE_BLOCKS:u64 = 1;
/// requires segwit support
pub const SERVICE_WITNESS:u64 =  1 << 3;
/// require filters
pub const SERVICE_FILTERS:u64 = 1 << 6;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct PeerId {
    network: &'static str,
    // mio token used in networking
    token: Token
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}-{}", self.network, self.token.0)?;
        Ok(())
    }
}
type PeerMap<Message> = HashMap<PeerId, Mutex<Peer<Message>>>;

pub enum P2PControl<Message: Clone> {
    Send(PeerId, Message),
    Broadcast(Message),
    Ban(PeerId, u32),
    Disconnect(PeerId),
    Height(u32),
    Bind(IpSocketAddress)
}

impl<Message: Send + Sync + Clone> P2PControlSender<Message> {
    fn new (sender: mpsc::Sender<P2PControl<Message>>, peers: Arc<RwLock<PeerMap<Message>>>, back_pressure: usize) -> P2PControlSender<Message> {
        P2PControlSender { sender: Arc::new(Mutex::new(sender)), peers, back_pressure }
    }

    pub fn send (&self, control: P2PControl<Message>) {
        self.sender.lock().unwrap().send(control).expect("P2P control send failed");
    }

    pub fn send_network (&self, peer: PeerId, msg: Message) {
        self.send(P2PControl::Send(peer, msg))
    }

    pub fn send_random_network (&self, msg: Message) -> Option<PeerId> {
        let peers = self.peers.read().unwrap().keys().cloned().collect::<Vec<PeerId>>();
        if peers.len() > 0 {
            let peer = peers[(thread_rng().next_u32() % peers.len() as u32) as usize];
            self.send(P2PControl::Send(peer, msg));
            return Some(peer);
        }
        None
    }

    pub fn broadcast (&self, msg: Message) {
        self.send(P2PControl::Broadcast(msg))
    }

    pub fn ban(&self, peer: PeerId, increment: u32) {
        println!("increase ban score with {} peer={}", increment, peer);
        self.send(P2PControl::Ban(peer, increment))
    }

    pub fn peer_version (&self, peer: PeerId) -> Option<VersionCarrier> {
        if let Some(peer) = self.peers.read().unwrap().get(&peer) {
            let locked_peer = peer.lock().unwrap();
            return locked_peer.version.clone();
        }
        None
    }

    pub fn peers (&self) -> Vec<PeerId> {
        self.peers.read().unwrap().keys().cloned().collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct P2PControlSender<Message: Clone> {
    sender: Arc<Mutex<mpsc::Sender<P2PControl<Message>>>>,
    peers: Arc<RwLock<PeerMap<Message>>>,
    pub back_pressure: usize
}

/// A message from network to downstream
#[derive(Clone)]
pub enum PeerMessage<Message: Send + Sync + Clone> {
    Outgoing(Message),
    Incoming(PeerId, Message),
    Connected(PeerId, Option<IpSocketAddress>),
    Disconnected(PeerId, bool) // true if banned
}

#[derive(Clone)]
pub enum PeerSource {
    Outgoing(IpSocketAddress),
    Incoming(Arc<TcpListener>)
}

/// a map of peer id to peers
pub type PeerMessageReceiver<Message> = mpsc::Receiver<PeerMessage<Message>>;

#[derive(Clone)]
pub struct PeerMessageSender<Message: Send + Sync + Clone> {
    sender: Option<Arc<Mutex<mpsc::Sender<PeerMessage<Message>>>>>
}

impl<Message: Send + Sync + Clone> PeerMessageSender<Message> {
    pub fn new (sender: mpsc::Sender <PeerMessage<Message>>) -> PeerMessageSender<Message> {
        PeerMessageSender { sender: Some(Arc::new(Mutex::new(sender))) }
    }

    pub fn dummy () -> PeerMessageSender<Message> {
        PeerMessageSender{ sender: None }
    }

    pub async fn send (&self, msg: PeerMessage<Message>) {
        if let Some(ref sender) = self.sender {
            sender.lock().unwrap().send(msg).await.expect("P2P message send failed");
        }
    }
}

#[derive(Clone)]
pub struct VersionCarrier {
    /// The P2P network protocol version
    pub version: u32,
    /// A bitmask describing the services supported by this node
    pub services: u64,
    /// The time at which the `version` message was sent
    pub timestamp: u64,
    /// The network address of the peer receiving the message
    pub receiver: Address,
    /// The network address of the peer sending the message
    pub sender: Address,
    /// A random nonce used to detect loops in the network
    pub nonce: u64,
    /// A string describing the peer's software
    pub user_agent: String,
    /// The height of the maximum-work blockchain that the peer is aware of
    pub start_height: u32,
    /// Whether the receiving peer should relay messages to the sender; used
    /// if the sender is bandwidth-limited and would like to support bloom
    /// filtering. Defaults to true.
    pub relay: bool
}

pub trait Command {
    fn command(&self)->CommandString;
}

impl Command for RawNetworkMessage {
    fn command(&self) -> CommandString   {
        self.command()
    }
}

pub trait Version {
    fn is_verack(&self) ->bool;
    fn is_version(&self) -> Option<VersionCarrier>;
}

impl Version for NetworkMessage {
    fn is_version(&self) -> Option<VersionCarrier> {
        match self {
            NetworkMessage::Version(v) => {
                Some(VersionCarrier {
                    version: v.version,
                    services: v.services,
                    timestamp: v.timestamp as u64,
                    receiver: v.receiver.clone(),
                    sender: v.sender.clone(),
                    nonce: v.nonce,
                    user_agent: v.user_agent.clone(),
                    start_height: v.start_height as u32,
                    relay: v.relay
                })
            },
            _ => None
        }
    }

    fn is_verack(&self) -> bool {
        match self {
            NetworkMessage::Verack => true,
            _ => false
        }
    }

}

pub trait P2PConfig<Message: Version + Send + Sync + 'static, Envelope: Command + Send + Sync + 'static> {
    fn version (&self, remote: &IpSocketAddress, max_protocol_version: u32) -> Message;
    fn nonce(&self) -> u64;
    fn magic(&self) -> u32;
    fn user_agent(&self) -> &str;
    fn get_height(&self) -> u32;
    fn set_height(&self, height: u32);
    fn max_protocol_version(&self) -> u32;
    fn min_protocol_version(&self) -> u32;
    fn verack(&self) -> Message;
    fn wrap(&self, m: Message) -> Envelope;
    fn unwrap(&self, e: Envelope) -> Result<Message, io::Error>;
    fn encode(&self, item: &Envelope, dst: &mut Buffer) -> Result<(), io::Error>;
    fn decode(&self, src: &mut Buffer) -> Result<Option<Envelope>, io::Error>;
}

struct PassThroughBufferReader<'a> {
    buffer: &'a mut Buffer
}

impl<'a> io::Read for PassThroughBufferReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.buffer.read(buf)
    }
}

impl P2PConfig<NetworkMessage, RawNetworkMessage> for BitcoinP2PConfig {
    // compile this node's version message for outgoing connections
    fn version (&self, remote: &IpSocketAddress, max_protocol_version: u32) -> NetworkMessage {
        // now in unix time
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        let services = if !self.server {
            0
        } else {
            SERVICE_BLOCKS + SERVICE_WITNESS +
                // announce that this node is capable of serving BIP157 messages
                SERVICE_FILTERS
        };

        // build message
        NetworkMessage::Version(VersionMessage {
            version: min(max_protocol_version, self.max_protocol_version),
            services,
            timestamp,
            receiver: Address::new(remote, 1),
            // sender is only dummy
            sender: Address::new(remote, 1),
            nonce: self.nonce,
            user_agent: self.user_agent.clone(),
            start_height: self.height.load(Ordering::Relaxed) as i32,
            relay: true,
        })
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn magic(&self) -> u32 {
        self.network.magic()
    }

    fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }

    fn get_height(&self) -> u32 {
        self.height.load(Ordering::Relaxed) as u32
    }

    fn set_height(&self, height: u32) {
        self.height.store (height as usize, Ordering::Relaxed)
    }

    fn max_protocol_version(&self) -> u32 {
        self.max_protocol_version
    }

    fn min_protocol_version(&self) -> u32 {
        70001
    }


    fn verack(&self) -> NetworkMessage {
        NetworkMessage::Verack
    }

    fn wrap(&self, m: NetworkMessage) -> RawNetworkMessage {
        RawNetworkMessage{magic: self.network.magic(), payload: m, payload_len: , checksum: todo!()  }
    }

    fn unwrap(&self, e: RawNetworkMessage) -> Result<NetworkMessage, io::Error> {
        Ok(e.payload())
    }

    // encode a message in Bitcoin's wire format extending the given buffer
    fn encode(&self, item: &RawNetworkMessage, dst: &mut Buffer) -> Result<(), io::Error> {
        dst.write_all(serialize(item).as_slice())
    }

    // decode a message from the buffer if possible
    fn decode(&self, src: &mut Buffer) -> Result<Option<RawNetworkMessage>, io::Error> {
        // attempt to decode
        let passthrough = PassThroughBufferReader{buffer: src};
        let decode: Result<RawNetworkMessage, encode::Error> =
            Decodable::consensus_decode(&mut passthrough);

        match decode {
            Ok(m) => {
                // success: free the read data in buffer and return the message
                src.commit();
                Ok(Some(m))
            }
            Err(encode::Error::Io(e)) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    // need more data, rollback and retry after additional read
                    src.rollback();
                    return Ok(None)
                } else {
                    eprintln!("{:?}", e);
                    src.commit();
                    return Err(e);
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
                src.commit();
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
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



pub struct P2P<Message: Version + Send + Sync + Clone + 'static,
    Envelope: Command + Send + Sync + 'static,
    Config: P2PConfig<Message, Envelope> + Send + Sync + 'static> {
     // sender to the dispatcher of incoming messages
     dispatcher: PeerMessageSender<Message>,
     // network specific conf
     pub config: Config,
     // The collection of connected peers
     peers: Arc<RwLock<PeerMap<Message>>>,
     // The poll object of the async IO layer (mio)
     // access to this is shared by P2P and Peer
     poll: Arc<Poll>,
     // next peer id
     // atomic only for interior mutability
     next_peer_id: AtomicUsize,
     // waker
     waker: Arc<Mutex<HashMap<PeerId, Waker>>>,
     // server
     listener: Mutex<HashMap<Token, Arc<TcpListener>>>,
     e: PhantomData<Envelope>
}


struct Peer<Message> {
    /// the peer's id for log messages
    pub pid: PeerId,
    // the event poller, shared with P2P, needed here to register for events
    poll: Arc<Poll>,
    // the connection to remote peer
    stream: TcpStream,
    // temporary buffer for not yet completely read incoming messages
    read_buffer: Buffer,
    // temporary buffer for not yet completely written outgoing messages
    write_buffer: Buffer,
    // did the remote peer already sent a verack?
    got_verack: bool,
    /// the version message the peer sent to us at connect
    pub version: Option<VersionCarrier>,
    // channel into the event processing loop for outgoing messages
    sender: mpsc::Sender<Message>,
    // channel into the event processing loop for outgoing messages
    receiver: mpsc::Receiver<Message>,
    // is registered for write?
    writeable: AtomicBool,
    // connected and handshake complete?
    connected: bool,
    // ban score
    ban: u32,
    // outgoing or incoming connection
    outgoing: bool
}