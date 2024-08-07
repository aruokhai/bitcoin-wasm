use std::sync::Arc;

use uuid::Uuid;
use wasi::{clocks::wall_clock, random};
use web5::{credentials::verifiable_credential_1_1::{CredentialSubject, Issuer, VerifiableCredential, BASE_CONTEXT, BASE_TYPE}, crypto::{self, jwk::Jwk, key_managers::in_memory_key_manager::InMemoryKeyManager}, dids::{bearer_did::BearerDid, methods::did_dht::DidDht}};

mod bindings;
mod web5;

fn main() {
    println!("Hello, world!");
    let jwk = create_jwk();
    verify_did(jwk.clone());
    verify_credentials(jwk);
}

fn create_jwk() -> Jwk {
    let jwk = web5::crypto::dsa::ed25519::Ed25519Generator::generate();
    return jwk;
}

fn verify_did(jwk: Jwk) {
    let new_jk = jwk.clone();
    let ecdsa_signer = Arc::new(crypto::dsa::ed25519::Ed25519Signer::new(jwk));
    let did = DidDht::from_identity_key(new_jk).unwrap();
    let url = did.clone().did.url;
    let _ = did.publish(ecdsa_signer).unwrap();
    let resolved_did = DidDht::resolve(url.as_str());
    assert_eq!(url, resolved_did.clone().document.unwrap().id);
}

fn verify_credentials(jwk: Jwk) {
    let key_manager = InMemoryKeyManager::new();
        let public_jwk = key_manager
            .import_private_jwk(jwk)
            .unwrap();
    let did = DidDht::from_identity_key(public_jwk).unwrap();
    let bearer_did =  BearerDid::new(&did.did.uri, Arc::new(key_manager)).unwrap();
    let now = wasi::clocks::wall_clock::now();
    let random_number1 = random::random::get_random_u64();
    let random_number2 = random::random::get_random_u64();
    let uuid = Uuid::from_u64_pair(random_number1, random_number2).to_string();
    let vc = VerifiableCredential::new(
        format!("urn:vc:uuid:{0}", uuid),
        vec![BASE_CONTEXT.to_string()],
        vec![BASE_TYPE.to_string()],
        Issuer::String(bearer_did.did.uri.clone()),
        now,
        Some(wall_clock::Datetime{ seconds: now.seconds + (20 * 365 * 24 * 60 * 60) as u64, nanoseconds: 0 }), // now + 20 years
        CredentialSubject {
            id: bearer_did.did.uri.clone(),
            ..Default::default()
        },
    );
    let vc_jwt = vc.sign(&bearer_did).unwrap();
    assert_ne!(String::default(), vc_jwt);
    VerifiableCredential::verify(&vc_jwt).unwrap();
    
}