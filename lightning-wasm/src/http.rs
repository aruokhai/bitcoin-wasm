use crate::disk::{self, INBOUND_PAYMENTS_FNAME, OUTBOUND_PAYMENTS_FNAME};
use crate::hex_utils;
use crate::{
	ChannelManager, HTLCStatus, InboundPaymentInfoStorage, MillisatAmount, NetworkGraph,
	OnionMessenger, OutboundPaymentInfoStorage, PaymentInfo, PeerManager,
};
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::Hash;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::PublicKey;
use lightning::ln::channelmanager::{PaymentId, RecipientOnionFields, Retry};
use lightning::ln::msgs::SocketAddress;
use lightning::ln::{ChannelId, PaymentHash, PaymentPreimage};
use lightning::offers::offer::{self, Offer};
use lightning::onion_message::messenger::Destination;
use lightning::onion_message::packet::OnionMessageContents;
use lightning::routing::gossip::NodeId;
use lightning::routing::router::{PaymentParameters, RouteParameters};
use lightning::sign::{EntropySource, KeysManager};
use lightning::util::config::{ChannelHandshakeConfig, ChannelHandshakeLimits, UserConfig};
use lightning::util::persist::KVStore;
use lightning::util::ser::{Writeable, Writer};
use lightning_invoice::payment::payment_parameters_from_invoice;
use lightning_invoice::payment::payment_parameters_from_zero_amount_invoice;
use lightning_invoice::{utils, Bolt11Invoice, Currency};
use lightning_persister::fs_store::FilesystemStore;
use warp::Filter;
use std::env;
use std::io;
use std::io::Write;
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Serialize,Deserialize};

pub(crate) struct LdkUserInfo {
	pub(crate) bitcoind_rpc_username: String,
	pub(crate) bitcoind_rpc_password: String,
	pub(crate) bitcoind_rpc_port: u16,
	pub(crate) bitcoind_rpc_host: String,
	pub(crate) ldk_storage_dir_path: String,
	pub(crate) ldk_peer_listening_port: u16,
	pub(crate) ldk_announced_listen_addr: Vec<SocketAddress>,
	pub(crate) ldk_announced_node_name: [u8; 32],
	pub(crate) network: Network,
}

#[derive(Serialize, Deserialize)]
struct OpenChannelRequest {
    peer_pub_key: String,
    peer_ip_addr: String,
    amount_sats: u64,
}

#[derive(Serialize, Deserialize)]
struct OpenPeerRequest {
    peer_pub_key: String,
    peer_ip_addr: String,
}

#[derive(Debug)]
struct UserOnionMessageContents {
	tlv_type: u64,
	data: Vec<u8>,
}

impl OnionMessageContents for UserOnionMessageContents {
	fn tlv_type(&self) -> u64 {
		self.tlv_type
	}
}

impl Writeable for UserOnionMessageContents {
	fn write<W: Writer>(&self, w: &mut W) -> Result<(), std::io::Error> {
		w.write_all(&self.data)
	}
}

pub(crate) fn create_client_server(
	peer_manager: Arc<PeerManager>, channel_manager: Arc<ChannelManager>,
	keys_manager: Arc<KeysManager>, network_graph: Arc<NetworkGraph>,
	onion_messenger: Arc<OnionMessenger>, inbound_payments: Arc<Mutex<InboundPaymentInfoStorage>>,
	outbound_payments: Arc<Mutex<OutboundPaymentInfoStorage>>, ldk_data_dir: String,
	network: Network, logger: Arc<disk::FilesystemLogger>, fs_store: Arc<FilesystemStore>,
) {
	println!(
		"LDK startup successful. Starting Server."
	);
	println!("LDK logs are available at <your-supplied-ldk-data-dir-path>/.ldk/logs");
	println!("Local Node ID is {}.", channel_manager.get_our_node_id());

    // GET /
    let help = warp::get()
    .and(warp::path::end())
    .map(|| "Try POSTing data to /echo such as: `curl localhost:8080/echo -XPOST -d 'hello world'`\n");

    // POST /openchannel
    let openchannel = warp::post()
        .and(warp::path("openchannel"))
        .and(warp::body::json::<OpenChannelRequest>())
        .and_then(|body: OpenChannelRequest| async move {
            let (pubkey, peer_addr) = parse_peer_info(body.peer_pub_key,body.peer_ip_addr)
                .map_err(|err| warp::reject::not_found())?;
            let (mut announce_channel, mut with_anchors) = (false, false);
            if open_channel(
                pubkey,
                chan_amt_sat,
                announce_channel,
                with_anchors,
                channel_manager.clone(),
            )
            .is_ok()
            {
                let peer_data_path = format!("{}/channel_peer_data", ldk_data_dir.clone());
                let _ = disk::persist_channel_peer(
                    Path::new(&peer_data_path),
                    peer_pubkey_and_ip_addr,
                );
                return  Ok(())
            }
        });

    // POST /connectpeer
    let connectpeer = warp::post()
    .and(warp::path("connectpeer"))
    .and(warp::body::json::<OpenPeerRequest>())
    .and_then(|body: OpenPeerRequest| async move {
        let (pubkey, peer_addr) = parse_peer_info(body.peer_pub_key,body.peer_ip_addr)
            .map_err(|err| warp::reject::not_found())?;
        if tokio::runtime::Handle::current()
			.block_on(connect_peer_if_necessary(
				pubkey,
				peer_addr,
				peer_manager.clone(),
			))
			.is_ok()
		    {
		    	Ok(pubkey)
		    }
    });
  let routes = help.or(openchannel).or(connectpeer);

  let (addr, server) = warp::serve(routes)
    .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
         println!("shutdown")
    });

    // Spawn the server into a runtime
    tokio::task::spawn(server);

	
}


fn node_info(channel_manager: &Arc<ChannelManager>, peer_manager: &Arc<PeerManager>) {
	println!("\t{{");
	println!("\t\t node_pubkey: {}", channel_manager.get_our_node_id());
	let chans = channel_manager.list_channels();
	println!("\t\t num_channels: {}", chans.len());
	println!("\t\t num_usable_channels: {}", chans.iter().filter(|c| c.is_usable).count());
	let local_balance_msat = chans.iter().map(|c| c.balance_msat).sum::<u64>();
	println!("\t\t local_balance_msat: {}", local_balance_msat);
	println!("\t\t num_peers: {}", peer_manager.get_peer_node_ids().len());
	println!("\t}},");
}

fn list_peers(peer_manager: Arc<PeerManager>) {
	println!("\t{{");
	for (pubkey, _) in peer_manager.get_peer_node_ids() {
		println!("\t\t pubkey: {}", pubkey);
	}
	println!("\t}},");
}

fn list_channels(channel_manager: &Arc<ChannelManager>, network_graph: &Arc<NetworkGraph>) {
	print!("[");
	for chan_info in channel_manager.list_channels() {
		println!("");
		println!("\t{{");
		println!("\t\tchannel_id: {},", chan_info.channel_id);
		if let Some(funding_txo) = chan_info.funding_txo {
			println!("\t\tfunding_txid: {},", funding_txo.txid);
		}

		println!(
			"\t\tpeer_pubkey: {},",
			hex_utils::hex_str(&chan_info.counterparty.node_id.serialize())
		);
		if let Some(node_info) = network_graph
			.read_only()
			.nodes()
			.get(&NodeId::from_pubkey(&chan_info.counterparty.node_id))
		{
			if let Some(announcement) = &node_info.announcement_info {
				println!("\t\tpeer_alias: {}", announcement.alias);
			}
		}

		if let Some(id) = chan_info.short_channel_id {
			println!("\t\tshort_channel_id: {},", id);
		}
		println!("\t\tis_channel_ready: {},", chan_info.is_channel_ready);
		println!("\t\tchannel_value_satoshis: {},", chan_info.channel_value_satoshis);
		println!("\t\toutbound_capacity_msat: {},", chan_info.outbound_capacity_msat);
		if chan_info.is_usable {
			println!("\t\tavailable_balance_for_send_msat: {},", chan_info.outbound_capacity_msat);
			println!("\t\tavailable_balance_for_recv_msat: {},", chan_info.inbound_capacity_msat);
		}
		println!("\t\tchannel_can_send_payments: {},", chan_info.is_usable);
		println!("\t\tpublic: {},", chan_info.is_public);
		println!("\t}},");
	}
	println!("]");
}

fn list_payments(
	inbound_payments: &InboundPaymentInfoStorage, outbound_payments: &OutboundPaymentInfoStorage,
) {
	print!("[");
	for (payment_hash, payment_info) in &inbound_payments.payments {
		println!("");
		println!("\t{{");
		println!("\t\tamount_millisatoshis: {},", payment_info.amt_msat);
		println!("\t\tpayment_hash: {},", payment_hash);
		println!("\t\thtlc_direction: inbound,");
		println!(
			"\t\thtlc_status: {},",
			match payment_info.status {
				HTLCStatus::Pending => "pending",
				HTLCStatus::Succeeded => "succeeded",
				HTLCStatus::Failed => "failed",
			}
		);

		println!("\t}},");
	}

	for (payment_hash, payment_info) in &outbound_payments.payments {
		println!("");
		println!("\t{{");
		println!("\t\tamount_millisatoshis: {},", payment_info.amt_msat);
		println!("\t\tpayment_hash: {},", payment_hash);
		println!("\t\thtlc_direction: outbound,");
		println!(
			"\t\thtlc_status: {},",
			match payment_info.status {
				HTLCStatus::Pending => "pending",
				HTLCStatus::Succeeded => "succeeded",
				HTLCStatus::Failed => "failed",
			}
		);

		println!("\t}},");
	}
	println!("]");
}

pub(crate) async fn connect_peer_if_necessary(
	pubkey: PublicKey, peer_addr: SocketAddr, peer_manager: Arc<PeerManager>,
) -> Result<(), ()> {
	for (node_pubkey, _) in peer_manager.get_peer_node_ids() {
		if node_pubkey == pubkey {
			return Ok(());
		}
	}
	let res = do_connect_peer(pubkey, peer_addr, peer_manager).await;
	if res.is_err() {
		println!("ERROR: failed to connect to peer");
	}
	res
}

pub(crate) async fn do_connect_peer(
	pubkey: PublicKey, peer_addr: SocketAddr, peer_manager: Arc<PeerManager>,
) -> Result<(), ()> {
	match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, peer_addr).await
	{
		Some(connection_closed_future) => {
			let mut connection_closed_future = Box::pin(connection_closed_future);
			loop {
				tokio::select! {
					_ = &mut connection_closed_future => return Err(()),
					_ = tokio::time::sleep(Duration::from_millis(10)) => {},
				};
				if peer_manager.get_peer_node_ids().iter().find(|(id, _)| *id == pubkey).is_some() {
					return Ok(());
				}
			}
		}
		None => Err(()),
	}
}

fn do_disconnect_peer(
	pubkey: bitcoin::secp256k1::PublicKey, peer_manager: Arc<PeerManager>,
	channel_manager: Arc<ChannelManager>,
) -> Result<(), ()> {
	//check for open channels with peer
	for channel in channel_manager.list_channels() {
		if channel.counterparty.node_id == pubkey {
			println!("Error: Node has an active channel with this peer, close any channels first");
			return Err(());
		}
	}

	//check the pubkey matches a valid connected peer
	let peers = peer_manager.get_peer_node_ids();
	if !peers.iter().any(|(pk, _)| &pubkey == pk) {
		println!("Error: Could not find peer {}", pubkey);
		return Err(());
	}

	peer_manager.disconnect_by_node_id(pubkey);
	Ok(())
}

fn open_channel(
	peer_pubkey: PublicKey, channel_amt_sat: u64, announced_channel: bool, with_anchors: bool,
	channel_manager: Arc<ChannelManager>,
) -> Result<(), ()> {
	let config = UserConfig {
		channel_handshake_limits: ChannelHandshakeLimits {
			// lnd's max to_self_delay is 2016, so we want to be compatible.
			their_to_self_delay: 2016,
			..Default::default()
		},
		channel_handshake_config: ChannelHandshakeConfig {
			announced_channel,
			negotiate_anchors_zero_fee_htlc_tx: with_anchors,
			..Default::default()
		},
		..Default::default()
	};

	match channel_manager.create_channel(peer_pubkey, channel_amt_sat, 0, 0, None, Some(config)) {
		Ok(_) => {
			println!("EVENT: initiated channel with peer {}. ", peer_pubkey);
			return Ok(());
		}
		Err(e) => {
			println!("ERROR: failed to open channel: {:?}", e);
			return Err(());
		}
	}
}

fn send_payment(
	channel_manager: &ChannelManager, invoice: &Bolt11Invoice, required_amount_msat: Option<u64>,
	outbound_payments: &mut OutboundPaymentInfoStorage, fs_store: Arc<FilesystemStore>,
) {
	let payment_id = PaymentId((*invoice.payment_hash()).to_byte_array());
	let payment_secret = Some(*invoice.payment_secret());
	let zero_amt_invoice =
		invoice.amount_milli_satoshis().is_none() || invoice.amount_milli_satoshis() == Some(0);
	let pay_params_opt = if zero_amt_invoice {
		if let Some(amt_msat) = required_amount_msat {
			payment_parameters_from_zero_amount_invoice(invoice, amt_msat)
		} else {
			println!("Need an amount for the given 0-value invoice");
			print!("> ");
			return;
		}
	} else {
		if required_amount_msat.is_some() && invoice.amount_milli_satoshis() != required_amount_msat
		{
			println!(
				"Amount didn't match invoice value of {}msat",
				invoice.amount_milli_satoshis().unwrap_or(0)
			);
			print!("> ");
			return;
		}
		payment_parameters_from_invoice(invoice)
	};
	let (payment_hash, recipient_onion, route_params) = match pay_params_opt {
		Ok(res) => res,
		Err(e) => {
			println!("Failed to parse invoice");
			print!("> ");
			return;
		}
	};
	outbound_payments.payments.insert(
		payment_id,
		PaymentInfo {
			preimage: None,
			secret: payment_secret,
			status: HTLCStatus::Pending,
			amt_msat: MillisatAmount(invoice.amount_milli_satoshis()),
		},
	);
	fs_store.write("", "", OUTBOUND_PAYMENTS_FNAME, &outbound_payments.encode()).unwrap();

	match channel_manager.send_payment(
		payment_hash,
		recipient_onion,
		payment_id,
		route_params,
		Retry::Timeout(Duration::from_secs(10)),
	) {
		Ok(_) => {
			let payee_pubkey = invoice.recover_payee_pub_key();
			let amt_msat = invoice.amount_milli_satoshis().unwrap();
			println!("EVENT: initiated sending {} msats to {}", amt_msat, payee_pubkey);
			print!("> ");
		}
		Err(e) => {
			println!("ERROR: failed to send payment: {:?}", e);
			print!("> ");
			outbound_payments.payments.get_mut(&payment_id).unwrap().status = HTLCStatus::Failed;
			fs_store.write("", "", OUTBOUND_PAYMENTS_FNAME, &outbound_payments.encode()).unwrap();
		}
	};
}

fn keysend<E: EntropySource>(
	channel_manager: &ChannelManager, payee_pubkey: PublicKey, amt_msat: u64, entropy_source: &E,
	outbound_payments: &mut OutboundPaymentInfoStorage, fs_store: Arc<FilesystemStore>,
) {
	let payment_preimage = PaymentPreimage(entropy_source.get_secure_random_bytes());
	let payment_id = PaymentId(Sha256::hash(&payment_preimage.0[..]).to_byte_array());

	let route_params = RouteParameters::from_payment_params_and_value(
		PaymentParameters::for_keysend(payee_pubkey, 40, false),
		amt_msat,
	);
	outbound_payments.payments.insert(
		payment_id,
		PaymentInfo {
			preimage: None,
			secret: None,
			status: HTLCStatus::Pending,
			amt_msat: MillisatAmount(Some(amt_msat)),
		},
	);
	fs_store.write("", "", OUTBOUND_PAYMENTS_FNAME, &outbound_payments.encode()).unwrap();
	match channel_manager.send_spontaneous_payment_with_retry(
		Some(payment_preimage),
		RecipientOnionFields::spontaneous_empty(),
		payment_id,
		route_params,
		Retry::Timeout(Duration::from_secs(10)),
	) {
		Ok(_payment_hash) => {
			println!("EVENT: initiated sending {} msats to {}", amt_msat, payee_pubkey);
			print!("> ");
		}
		Err(e) => {
			println!("ERROR: failed to send payment: {:?}", e);
			print!("> ");
			outbound_payments.payments.get_mut(&payment_id).unwrap().status = HTLCStatus::Failed;
			fs_store.write("", "", OUTBOUND_PAYMENTS_FNAME, &outbound_payments.encode()).unwrap();
		}
	};
}

fn get_invoice(
	amt_msat: u64, inbound_payments: &mut InboundPaymentInfoStorage,
	channel_manager: &ChannelManager, keys_manager: Arc<KeysManager>, network: Network,
	expiry_secs: u32, logger: Arc<disk::FilesystemLogger>,
) {
	let currency = match network {
		Network::Bitcoin => Currency::Bitcoin,
		Network::Regtest => Currency::Regtest,
		Network::Signet => Currency::Signet,
		Network::Testnet | _ => Currency::BitcoinTestnet,
	};
	let invoice = match utils::create_invoice_from_channelmanager(
		channel_manager,
		keys_manager,
		logger,
		currency,
		Some(amt_msat),
		"ldk-tutorial-node".to_string(),
		expiry_secs,
		None,
	) {
		Ok(inv) => {
			println!("SUCCESS: generated invoice: {}", inv);
			inv
		}
		Err(e) => {
			println!("ERROR: failed to create invoice: {:?}", e);
			return;
		}
	};

	let payment_hash = PaymentHash(invoice.payment_hash().to_byte_array());
	inbound_payments.payments.insert(
		payment_hash,
		PaymentInfo {
			preimage: None,
			secret: Some(invoice.payment_secret().clone()),
			status: HTLCStatus::Pending,
			amt_msat: MillisatAmount(Some(amt_msat)),
		},
	);
}

fn close_channel(
	channel_id: [u8; 32], counterparty_node_id: PublicKey, channel_manager: Arc<ChannelManager>,
) {
	match channel_manager.close_channel(&ChannelId(channel_id), &counterparty_node_id) {
		Ok(()) => println!("EVENT: initiating channel close"),
		Err(e) => println!("ERROR: failed to close channel: {:?}", e),
	}
}

fn force_close_channel(
	channel_id: [u8; 32], counterparty_node_id: PublicKey, channel_manager: Arc<ChannelManager>,
) {
	match channel_manager
		.force_close_broadcasting_latest_txn(&ChannelId(channel_id), &counterparty_node_id)
	{
		Ok(()) => println!("EVENT: initiating channel force-close"),
		Err(e) => println!("ERROR: failed to force-close channel: {:?}", e),
	}
}

pub(crate) fn parse_peer_info(peer_pubkey: String,
	peer_addr: String,
) -> Result<(PublicKey, SocketAddr), std::io::Error> {
	let peer_addr = peer_addr.to_socket_addrs().map(|mut r| r.next());
	if peer_addr.is_err() || peer_addr.as_ref().unwrap().is_none() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"ERROR: couldn't parse pubkey@host:port into a socket address",
		));
	}

	let pubkey = hex_utils::to_compressed_pubkey(&peer_pubkey);
	if pubkey.is_none() {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Other,
			"ERROR: unable to parse given pubkey for node",
		));
	}

	Ok((pubkey.unwrap(), peer_addr.unwrap().unwrap()))
}
