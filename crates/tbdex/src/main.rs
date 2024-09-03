use std::{fmt::format, sync::Arc};

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

const DEFAULT_PROTOCOL_VERSION: &str = "1.0";

fn get_utc_now() -> String {
    let utc_now = clocks::wall_clock::now();
    return chrono::DateTime::from_timestamp(utc_now.seconds as i64, utc_now.nanoseconds).unwrap().to_rfc3339();
}

fn main() {
    println!("Hello, world!");
    // let jwk = create_jwk();
    // verify_did(jwk.clone());
    // verify_credentials(jwk);
    tbdex_test();
}

// fn create_jwk() -> Jwk {
//     let jwk = web5::crypto::dsa::ed25519::Ed25519Generator::generate();
//     return jwk;
// }

// fn verify_did(jwk: Jwk) {
//     let new_jk = jwk.clone();
//     let ecdsa_signer = Arc::new(crypto::dsa::ed25519::Ed25519Signer::new(jwk));
//     let did = DidDht::from_identity_key(new_jk).unwrap();
//     let url = did.clone().did.url;
//     let _ = did.publish(ecdsa_signer).unwrap();
//     let resolved_did = DidDht::resolve(url.as_str());
//     assert_eq!(url, resolved_did.clone().document.unwrap().id);
// }

// fn verify_credentials(jwk: Jwk) {
//     let key_manager = InMemoryKeyManager::new();
//         let public_jwk = key_manager
//             .import_private_jwk(jwk)
//             .unwrap();
//     let did = DidDht::from_identity_key(public_jwk).unwrap();
//     let bearer_did =  BearerDid::new(&did.did.uri, Arc::new(key_manager)).unwrap();
//     let now = wasi::clocks::wall_clock::now();
//     let random_number1 = random::random::get_random_u64();
//     let random_number2 = random::random::get_random_u64();
//     let uuid = Uuid::from_u64_pair(random_number1, random_number2).to_string();
//     let vc = VerifiableCredential::new(
//         format!("urn:vc:uuid:{0}", uuid),
//         vec![BASE_CONTEXT.to_string()],
//         vec![BASE_TYPE.to_string()],
//         Issuer::String(bearer_did.did.uri.clone()),
//         now,
//         Some(wall_clock::Datetime{ seconds: now.seconds + (20 * 365 * 24 * 60 * 60) as u64, nanoseconds: 0 }), // now + 20 years
//         CredentialSubject {
//             id: bearer_did.did.uri.clone(),
//             ..Default::default()
//         },
//     );
//     let vc_jwt = vc.sign(&bearer_did).unwrap();
//     assert_ne!(String::default(), vc_jwt);
//     VerifiableCredential::verify(&vc_jwt).unwrap();
    
// }

fn tbdex_test() {
    let pfi_url = "did:dht:zz3m6ph36p1d8qioqfhp5dh5j6xn49cequ1yw9jnfxbz1uyfnddy";
    let offerings = get_offerings(pfi_url).unwrap();

    // get verifiable credential
    let jwk = web5::crypto::dsa::ed25519::Ed25519Generator::generate();
    let new_jk = jwk.clone();
    let ecdsa_signer = Arc::new(crypto::dsa::ed25519::Ed25519Signer::new(jwk.clone()));
    let ecdsa_verfier = Arc::new(crypto::dsa::ed25519::Ed25519Verifier::new(jwk.clone()));
    let did = DidDht::from_identity_key(new_jk).unwrap();
    let url = did.clone().did.url;
    let uri: String = did.clone().did.uri;
    println!("url is {}", url.clone());
    println!("uri is {}", uri.clone());

    let _ = did.publish(ecdsa_signer).unwrap();
    let credential_path =  format!("/vc?name=arrow&country=ZAR&did=${}",url );
    let credential_request = request(Method::Get, Scheme::Http, "localhost:9000", &credential_path, None, None, None, None, None).unwrap();
   let credentail = String::from_utf8(credential_request.body).unwrap();
   let verified_credentials_string  = offerings[0].clone().data.required_claims.unwrap().select_credentials(&vec![credentail.clone()]).unwrap();
   let verified_credential = VerifiableCredential::verify(&credentail).unwrap();
   // println!("my selected credential {:?}", offerings[0].clone().data.required_claims.unwrap());

   let my_offering = offerings[0].clone();
   let my_offering1 = offerings[0].clone();
   let payin_kind = my_offering.data.payin.methods[0].clone().kind;
   let payout_kind = my_offering.data.payout.methods[0].clone().kind;
   println!("Pay in and Payout kind {} , {}", payin_kind, payout_kind );
   let Offer_Data = CreateRfqData{offering_id: my_offering.metadata.id, payin:  CreateSelectedPayinMethod {
    amount: "10".to_string(),
    kind: my_offering.data.payin.methods[0].clone().kind,
    payment_details: None
   }, 
   payout: CreateSelectedPayoutMethod {
    kind: my_offering.data.payout.methods[0].clone().kind,
    payment_details: Some(serde_json::json!({"address": "abcd123s"}))
   }, 
    claims: verified_credentials_string};
   let mut rfq =  Rfq::create(&my_offering.metadata.from, &url, &Offer_Data, Some("1.0".to_owned()), None).unwrap();
   let inmemory_manager = InMemoryKeyManager::new();
   inmemory_manager.import_private_jwk(jwk.clone()).unwrap();
   let bearer_did = BearerDid::new(url.as_str(),Arc::new(inmemory_manager)).unwrap();
   rfq.sign(&bearer_did).unwrap();
   let exchange_created = create_exchange(&rfq, None).unwrap();


   let exchange_fetched = get_exchange_ids(&pfi_url, &bearer_did).unwrap();
   let myspecific_exchange_quote = exchange_fetched
    .iter()
    .flatten()
    .find( |message| {
        if let Message::Quote(_) =  message {
            return true;
        }
        return  false;
    }).unwrap();
   let myspecific_exchange_id = if let Message::Quote(order) =  myspecific_exchange_quote {
             Some(order.metadata.exchange_id.clone())
    } else {
        None
    }.unwrap();


   let mut new_order = Order::create(&pfi_url, &url, &myspecific_exchange_id, Some("1.0".to_owned()), None).unwrap();
   let _ = new_order.sign(&bearer_did).unwrap();
   let order = submit_order(&new_order).unwrap();
   loop {
    let exchange_fetched = get_exchange_ids(&pfi_url, &bearer_did).unwrap();
    let order_present = exchange_fetched
        .into_iter()
        .flatten()
        .any( | message| {
            if let Message::Close(close_message) =  message {
                if let  Some(true)= close_message.data.success{
                    println!("message is true");
                    return (close_message.metadata.exchange_id == myspecific_exchange_id)
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
   println!("success converting");
   
   
}