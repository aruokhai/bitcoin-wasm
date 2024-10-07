#[allow(warnings)]
mod bindings;
use std::{cell::RefCell};

use node::Node;
use bindings::exports::component::node::types::{Guest,Error, GuestClientNode, NodeConfig, TbdexConfig, OfferingBargain};
use bindings::component::tbdex::types::{Client, };




mod node;
mod p2p;
mod tcpsocket;
mod util;
mod wallet;
mod messages;

struct Component;

struct BitcoinNode {
    inner: RefCell<Node>,
    tbdex: Option<RefCell<Client>>,
}

impl GuestClientNode for BitcoinNode {
    fn get_balance(&self) -> Result<i64, Error> {
        return  self.inner.borrow_mut().get_balance().map_err(|_| Error::NetworkError);
    }

    fn get_conversion_offer(&self) -> Result<OfferingBargain, Error> {
        match &self.tbdex {
            Some(client) =>{
                let offer =client.borrow().get_offer().map_err(|_| Error::TbdexError)?;
                Ok( OfferingBargain { rate: offer.rate, fee: offer.fee, id: offer.id, estimated_settlement_time: offer.estimated_settlement_time})
            },
            None => {
                Err(Error::NoTbdx)
            },
        }
    }

    fn convert_amount(&self, amount: String, offer_id: String) -> Result<String, Error> {
        match &self.tbdex {
            Some(client) =>{
                // let address = self.inner.borrow().wallet.address.clone();
                let address = String::new();
                let res = client.borrow().convert(&offer_id, &amount, &address).map_err(|_| Error::TbdexError)?;
                Ok(res)
            },
            None => {
                Err(Error::NoTbdx)
            },
        }
    }

    fn new(config: NodeConfig, tbdx_config: Option<TbdexConfig>) -> Self {
        let tbdex = if let Some(config) = tbdx_config {
            let new_tbdex_client = Client::new(&config.pfi_uri, &config.vc_url, &config.acct_number);
            Some(RefCell::new(new_tbdex_client))
        } else {
            None
        };
        Self{ inner:  Node::new(config.into()).into(), tbdex}
    }
}

impl Guest for Component {
    
    type ClientNode  = BitcoinNode;
   
}

bindings::export!(Component with_types_in bindings);

