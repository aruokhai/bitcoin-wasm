use std::{collections::HashMap, sync::Arc};

use wasi::{clocks::monotonic_clock, http::types::{Method, Scheme}, io::poll::poll};

use crate::{http_client::{exchanges::{create_exchange, get_exchange_ids, submit_order}, offerings::get_offerings}, messages::{order::Order, rfq::{CreateRfqData, CreateSelectedPayinMethod, CreateSelectedPayoutMethod, Rfq}, Message}, request, resources::offering::{self, Offering}, web5::{self, crypto::{self, key_managers::in_memory_key_manager::InMemoryKeyManager}, dids::{bearer_did::BearerDid, methods::did_dht::DidDht}}};


pub struct Client {
    bearer_did: BearerDid,
    pfi_uri: String,
    credential: String,
    offerings: HashMap<String,Offering>,
    acct_number: String
}

enum ClientError {
    OfferNotFound
}
struct OfferingBargain {
    pub fee: Option<String>,
    pub estimated_settlement_time: u64,
    pub id: String,
    pub rate: String,
}

impl Client {

    pub fn new(pfi_uri: String, vc_url: String, acct_number: String ) -> Self {
        let jwk = web5::crypto::dsa::ed25519::Ed25519Generator::generate();
        let new_jk = jwk.clone();
        let ecdsa_signer = Arc::new(crypto::dsa::ed25519::Ed25519Signer::new(jwk.clone()));
        let did = DidDht::from_identity_key(new_jk).unwrap();
        let _ = did.publish(ecdsa_signer).unwrap();
        let inmemory_manager = InMemoryKeyManager::new();
        inmemory_manager.import_private_jwk(jwk.clone()).unwrap();
        let uri: String = did.clone().did.uri;
        let bearer_did = BearerDid::new(uri.as_str(),Arc::new(inmemory_manager)).unwrap();
        let credential_path =  format!("/vc?name=arrow&country=ZAR&did=${}",uri );
        let credential_request = request(Method::Get, Scheme::Https, &vc_url, &credential_path, None, None, None, None, None).unwrap();
       let credential = String::from_utf8(credential_request.body).unwrap();
       return Self {bearer_did, pfi_uri, credential, offerings: HashMap::new() ,acct_number }
    }

    pub fn get_offer(& mut self) -> Result<OfferingBargain, ClientError>{
        let offerings = get_offerings(&self.pfi_uri).unwrap();
        let btc_offering = offerings.iter().find(| offering | {
            offering.data.payin.currency_code == "USD".to_string()  && offering.data.payout.currency_code == "BTC".to_string()
        }).ok_or(ClientError::OfferNotFound)?;
        self.offerings.insert(btc_offering.metadata.id.clone(), btc_offering.clone());
        return  Ok(OfferingBargain {   rate: btc_offering.data.payout_units_per_payin_unit.clone() ,fee: btc_offering.data.payout.methods[0].fee.clone(),  id: btc_offering.metadata.id.clone(), estimated_settlement_time: btc_offering.data.payout.methods[0].estimated_settlement_time as u64 });
    }

    pub fn convert(& self , offer_id: String, amount: String, address: String) -> Result<String, ClientError> {
        let offering = self.offerings.get(&offer_id)
            .ok_or(ClientError::OfferNotFound)?;
        let payin_kind = offering.data.payin.methods[0].clone().kind;
        let payout_kind = offering.data.payout.methods[0].clone().kind;
        let offer_data = CreateRfqData{offering_id: offering.metadata.id.clone(), payin:  CreateSelectedPayinMethod {
         amount,
         kind: payin_kind,
         payment_details: Some(serde_json::json!({"accountNumber": self.acct_number, "routingNumber": "12345"}))
        }, 
        payout: CreateSelectedPayoutMethod {
         kind: payout_kind,
         payment_details: Some(serde_json::json!({"address": address}))
        }, 
         claims: vec![self.credential.clone()]
        };

        let mut rfq =  Rfq::create(&offering.metadata.from, &self.bearer_did.did.id, &offer_data, Some("1.0".to_owned()), None).unwrap();
        rfq.sign(&self.bearer_did).unwrap();
        create_exchange(&rfq, None).unwrap();

        let exchange_id: String = rfq.metadata.exchange_id;
        let mut new_order = Order::create(&self.pfi_uri, &self.bearer_did.did.id, &exchange_id, Some("1.0".to_owned()), None).unwrap();
        new_order.sign(&self.bearer_did).unwrap();
        submit_order(&new_order).unwrap();

        
        loop {
            let exchange_fetched = get_exchange_ids(&self.pfi_uri, &self.bearer_did).unwrap();
            let order_present = exchange_fetched
                .into_iter()
                .flatten()
                .any( | message| {
                    if let Message::Close(close_message) =  message {
                        if let  Some(true)= close_message.data.success{
                            return close_message.metadata.exchange_id == exchange_id
                        }
                    }
                    return  false;
                } );
            
            if order_present {
                break;
            }
            let pollable = monotonic_clock::subscribe_duration(5_000_000_000);
            poll(&[&pollable]);
        }

        return Ok(exchange_id);
         
    }
        
}
