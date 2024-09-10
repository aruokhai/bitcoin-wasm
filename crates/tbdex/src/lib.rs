use std::cell::RefCell;
#[allow(warnings)]
use std::{fmt::format, sync::Arc};

use client::Client;
use http_client::{exchanges::{create_exchange, get_exchange, get_exchange_ids, submit_order, Exchange}, offerings::get_offerings};
use messages::{order::Order, order_status::{OrderStatus, Status}, rfq::{CreateRfqData, CreateSelectedPayinMethod, CreateSelectedPayoutMethod, Rfq}, Message};
use request::request;
use uuid::Uuid;
use wasi::{clocks::{self, monotonic_clock, wall_clock}, http::types::{Method, Scheme}, io::poll::poll, random};
use web5::{credentials::verifiable_credential_1_1::{CredentialSubject, Issuer, VerifiableCredential, BASE_CONTEXT, BASE_TYPE}, crypto::{self, jwk::Jwk, key_managers::in_memory_key_manager::InMemoryKeyManager}, dids::{bearer_did::BearerDid, methods::did_dht::DidDht}};

mod bindings;
mod web5;
mod request;
mod json_schemas;
mod http;
mod json;
mod resources;
mod messages;
mod signature;
mod http_client;
mod client;

const DEFAULT_PROTOCOL_VERSION: &str = "1.0";

fn get_utc_now() -> String {
    let utc_now = clocks::wall_clock::now();
    return chrono::DateTime::from_timestamp(utc_now.seconds as i64, utc_now.nanoseconds).unwrap().to_rfc3339();
}

use bindings::exports::component::tbdex::{self, types::{Guest, OfferingBargain, Error, GuestClient}};
struct Component;

struct TbdexClient {
    inner: RefCell<Client>,
}

impl GuestClient for TbdexClient {
    fn get_offer(&self) -> Result<OfferingBargain, Error> {
        let bargain = self.inner.borrow_mut().get_offer().map_err(|_| {
            Error::OfferNotFound
        })?;
        let mapped_bargain = OfferingBargain {
            fee: bargain.fee,
            estimated_settlement_time: bargain.estimated_settlement_time,
            id: bargain.id,
            rate: bargain.rate
        };
        return  Ok(mapped_bargain);
    }

    fn convert(&self, offer_id: String, amount: String, address: String) -> Result<String, tbdex::types::Error> {
        let converted_details = self.inner.borrow_mut().convert(offer_id, amount, address).map_err(|_| {
            tbdex::types::Error::OfferNotFound
        })?;
        return Ok(converted_details);
    }
    
    fn new(pfi_uri: String, vc_url: String, acct_number: String) -> Self {
        let tbdex_client = client::Client::new(pfi_uri, vc_url, acct_number);
        return Self{ inner:  RefCell::new(tbdex_client)};
    }
}

impl Guest for Component {
    
    type Client = TbdexClient;
}

bindings::export!(Component with_types_in bindings);


// fn tbdex_test() {
//     let pfi_url = "did:dht:zkp5gbsqgzn69b3y5dtt5nnpjtdq6sxyukpzo68npsf79bmtb9zy";
//     let offerings = get_offerings(pfi_url).unwrap();
//     println!("my offerings are {:?}", offerings[3]);
//     // get verifiable credential
//     let jwk = web5::crypto::dsa::ed25519::Ed25519Generator::generate();
//     let new_jk = jwk.clone();
//     let ecdsa_signer = Arc::new(crypto::dsa::ed25519::Ed25519Signer::new(jwk.clone()));
//     let ecdsa_verfier = Arc::new(crypto::dsa::ed25519::Ed25519Verifier::new(jwk.clone()));
//     let did = DidDht::from_identity_key(new_jk).unwrap();
//     let uri: String = did.clone().did.uri;

//     let _ = did.publish(ecdsa_signer).unwrap();
//     let credential_path =  format!("/kcc?name=arrow&country=ZAR&did=${}",uri );
//     let credential_request = request(Method::Get, Scheme::Https, "mock-idv.tbddev.org", &credential_path, None, None, None, None, None).unwrap();
//     let credentail = String::from_utf8(credential_request.body).unwrap();
//     println!("credential is {}", credentail.clone());
//    let verified_credentials_string  = offerings[3].clone().data.required_claims.unwrap().select_credentials(&vec![credentail.clone()]).unwrap();
//    println!("verified credential is {:?}", verified_credentials_string.clone());

//    let verified_credential = VerifiableCredential::verify(&credentail).unwrap();

//    let my_offering = offerings[3].clone();
//    let payin_kind = my_offering.data.payin.methods[0].clone().kind;
//    let payout_kind = my_offering.data.payout.methods[0].clone().kind;
//    println!("Pay in and Payout kind {} , {}", payin_kind, payout_kind );
//    let Offer_Data = CreateRfqData{offering_id: my_offering.metadata.id, payin:  CreateSelectedPayinMethod {
//     amount: "10".to_string(),
//     kind: my_offering.data.payin.methods[0].clone().kind,
//     payment_details: Some(serde_json::json!({"accountNumber": "12345", "routingNumber": "12345"}))
//    }, 
//    payout: CreateSelectedPayoutMethod {
//     kind: my_offering.data.payout.methods[0].clone().kind,
//     payment_details: Some(serde_json::json!({"address": "abcd123s"}))
//    }, 
//     claims: vec![credentail.clone()]};
//    let mut rfq =  Rfq::create(&my_offering.metadata.from, &uri, &Offer_Data, Some("1.0".to_owned()), None).unwrap();
//    let inmemory_manager = InMemoryKeyManager::new();
//    inmemory_manager.import_private_jwk(jwk.clone()).unwrap();
//    let bearer_did = BearerDid::new(uri.as_str(),Arc::new(inmemory_manager)).unwrap();
//    rfq.sign(&bearer_did).unwrap();
//    let exchange_created = create_exchange(&rfq, None).unwrap();


//    let exchange_fetched = get_exchange_ids(&pfi_url, &bearer_did).unwrap();
//    let myspecific_exchange_quote = exchange_fetched
//     .iter()
//     .flatten()
//     .find( |message| {
//         if let Message::Quote(_) =  message {
//             return true;
//         }
//         return  false;
//     }).unwrap();
//    let myspecific_exchange_id = if let Message::Quote(order) =  myspecific_exchange_quote {
//              Some(order.metadata.exchange_id.clone())
//     } else {
//         None
//     }.unwrap();


//    let mut new_order = Order::create(&pfi_url, &uri, &myspecific_exchange_id, Some("1.0".to_owned()), None).unwrap();
//    let _ = new_order.sign(&bearer_did).unwrap();
//    let order = submit_order(&new_order).unwrap();
//    loop {
//     let exchange_fetched = get_exchange_ids(&pfi_url, &bearer_did).unwrap();
//     let order_present = exchange_fetched
//         .into_iter()
//         .flatten()
//         .any( | message| {
//             if let Message::Close(close_message) =  message {
//                 if let  Some(true)= close_message.data.success{
//                     println!("message is true");
//                     return (close_message.metadata.exchange_id == myspecific_exchange_id)
//                 }
//             }
//             return  false;
//         } );
    
//     if order_present {
//         break;
//     }
        
//     let pollable = monotonic_clock::subscribe_duration(5_000_000_000);
//     poll(&[&pollable]);

//    }
//    println!("success converting");
   
   
// }