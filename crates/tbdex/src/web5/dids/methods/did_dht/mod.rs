use bep44::Bep44Message;
use wasi::http::types::{Method, Scheme};
use simple_dns::Packet;

use super::{MethodError, Result};
use crate::web5::{
    request::request,
    crypto::{
        dsa::{
            ed25519::{self, Ed25519Verifier},
            Signer,
        },
        jwk::Jwk,
    },
    dids::{
        data_model::{document::Document, verification_method::VerificationMethod},
        did::Did,
        resolution::{
            resolution_metadata::{ResolutionMetadata, ResolutionMetadataError},
            resolution_result::ResolutionResult,
        },
    },
};
use std::sync::Arc;

pub mod bep44;
pub mod document_packet;

const JSON_WEB_KEY: &str = "JsonWebKey";
const DEFAULT_RELAY: &str = "diddht.tbddev.org";

fn create_identifier(identity_key_jwk: &Jwk) -> Result<String> {
    let pubkey_bytes = ed25519::public_jwk_extract_bytes(identity_key_jwk)?;
    let suffix = zbase32::encode_full_bytes(&pubkey_bytes);
    Ok(format!("did:dht:{}", suffix))
}

#[derive(Clone, Default, Debug)]
pub struct DidDht {
    pub did: Did,
    pub document: Document,
}

impl DidDht {
    pub fn from_identity_key(identity_key: Jwk) -> Result<Self> {
        if identity_key.crv != "Ed25519" {
            return Err(MethodError::DidCreationFailure(
                "Identity key must use Ed25519".to_string(),
            ));
        }
        let did_uri = create_identifier(&identity_key)?;
        let identity_key_verification_method = VerificationMethod {
            id: format!("{}#0", &did_uri),
            r#type: JSON_WEB_KEY.to_string(),
            controller: did_uri.clone(),
            public_key_jwk: identity_key,
        };

        let capability_delegation = vec![identity_key_verification_method.id.clone()];
        let capability_invocation = vec![identity_key_verification_method.id.clone()];
        let authentication = vec![identity_key_verification_method.id.clone()];
        let assertion_method = vec![identity_key_verification_method.id.clone()];
        let verification_methods = vec![identity_key_verification_method];

        // TODO maybe add additional verification methods and verification purposes
        // if let Some(additional_verification_methods) = additional_verification_methods {
        //     for vm_opts in additional_verification_methods {
        //         let verification_method = VerificationMethod {
        //             id: format!("{}#{}", did_uri, &vm_opts.public_key.compute_thumbprint().unwrap()), // TODO: don't unwrap
        //             r#type: JSON_WEB_KEY.to_string(),
        //             controller: "foo".to_string(),
        //             public_key_jwk: vm_opts.public_key,
        //         };

        //         for purpose in vm_opts.purposes {
        //             match purpose {
        //                 VerificationPurposes::Authentication => authentication.push(verification_method.id.clone()),
        //                 VerificationPurposes::AssertionMethod => assertion_method.push(verification_method.id.clone()),
        //                 VerificationPurposes::CapabilityInvocation => capability_invocation.push(verification_method.id.clone()),
        //                 VerificationPurposes::CapabilityDelegation => capability_delegation.push(verification_method.id.clone()),
        //                 VerificationPurposes::KeyAgreement => key_agreement.push(verification_method.id.clone()),
        //             }
        //         }

        //         verification_methods.push(verification_method);
        //     }
        // }

        Ok(Self {
            did: Did::new(&did_uri)?,
            document: Document {
                id: did_uri.clone(),
                verification_method: verification_methods,
                capability_delegation: Some(capability_delegation),
                capability_invocation: Some(capability_invocation),
                authentication: Some(authentication),
                assertion_method: Some(assertion_method),
                ..Default::default()
            },
        })
    }

    pub fn from_uri(uri: &str) -> Result<Self> {
        let resolution_result = DidDht::resolve(uri);
        match resolution_result.document {
            None => Err(match resolution_result.resolution_metadata.error {
                None => MethodError::ResolutionError(ResolutionMetadataError::InternalError),
                Some(e) => MethodError::ResolutionError(e),
            }),
            Some(document) => {
                let identifer = Did::new(uri)?;
                Ok(Self {
                    did: identifer,
                    document,
                })
            }
        }
    }

    pub fn resolve(uri: &str) -> ResolutionResult {
        let result: Result<ResolutionResult> = (|| {
            // check did method and decode id
            let did = Did::new(uri).map_err(|_| ResolutionMetadataError::InvalidDid)?;
            if did.method != "dht" {
                return Ok(ResolutionResult {
                    resolution_metadata: ResolutionMetadata {
                        error: Some(ResolutionMetadataError::MethodNotSupported),
                    },
                    ..Default::default()
                });
            }
            let identity_key = zbase32::decode_full_bytes_str(&did.id)
                .map_err(|_| ResolutionMetadataError::InvalidDid)?;
            let identity_key = ed25519::public_jwk_from_bytes(&identity_key)
                .map_err(|_| ResolutionMetadataError::InvalidDid)?;

            // construct http endpoint from gateway url and last part of did_uri
            let path = format!(
                "/{}",
                did.id.trim_start_matches('/')
            );

            // Make the GET request
            let response = request(Method::Get, Scheme::Https, &DEFAULT_RELAY.trim_end_matches('/'), &path, None, None, None, None, None).map_err(|_| ResolutionMetadataError::InternalError)?;

           
            // Check if the status is not 200
            let status = response.status;
            if status == 404 {
                return Err(ResolutionMetadataError::NotFound)?;
            } else if status != 200 {
                return Err(ResolutionMetadataError::InternalError)?;
            }

            // check http response status is 200 and body is nonempty
            let body = response.body;

            // Check if the body is empty
            if body.is_empty() {
                return Err(ResolutionMetadataError::NotFound)?;
            }

            // bep44 decode and verify response body bytes
            let body: Vec<u8> = body.into();
            let bep44_message = Bep44Message::decode(&body)
                .map_err(|_| ResolutionMetadataError::InvalidDidDocument)?;
            bep44_message
                .verify(&Ed25519Verifier::new(identity_key))
                .map_err(|_| ResolutionMetadataError::InvalidDidDocument)?;

            // convert bep44 decoded value from DNS packet to did doc
            let packet = Packet::parse(&bep44_message.v)
                .map_err(|_| ResolutionMetadataError::InvalidDidDocument)?;
            let document: Document = packet
                .try_into()
                .map_err(|_| ResolutionMetadataError::InvalidDidDocument)?;

            Ok(ResolutionResult {
                document: Some(document),
                ..Default::default()
            })
        })();

        match result {
            Ok(resolution_result) => resolution_result,
            Err(err) => ResolutionResult {
                resolution_metadata: ResolutionMetadata {
                    error: Some(match err {
                        MethodError::ResolutionError(e) => e,
                        _ => ResolutionMetadataError::InternalError,
                    }),
                },
                ..Default::default()
            },
        }
    }

    pub fn publish(&self, signer: Arc<dyn Signer>) -> Result<()> {
        let packet = self
            .document
            .to_packet()
            .map_err(|e| MethodError::DidPublishingFailure(e.to_string()))?;
        let packet_bytes = packet.build_bytes_vec().map_err(|_| {
            MethodError::DidPublishingFailure("Failed to serialize packet as bytes".to_string())
        })?;

        let bep44_message = Bep44Message::new(&packet_bytes, |payload| signer.sign(&payload))
            .map_err(|_| {
                MethodError::DidPublishingFailure(
                    "Failed to create bep44 message from packet".to_string(),
                )
            })?;
        let body = bep44_message.encode().map_err(|_| {
            MethodError::DidPublishingFailure(
                "Failed to serialize bep44 message as bytes".to_string(),
            )
        })?;

        let path = format!(
            "/{}",
            self.did.id.trim_start_matches('/')
        );

        // Make Post Request
        let response = request(Method::Put, Scheme::Https, &DEFAULT_RELAY.trim_end_matches('/'), &path, Some(body.as_slice()), Some(&[("Content-Type".into(), "application/octet-stream".as_bytes().to_vec())]), None, None, None)
            .map_err(|err| MethodError::DidPublishingFailure(err.to_string()))?;

        if response.status != 200 {
            return Err(MethodError::DidPublishingFailure(
                "Failed to PUT DID to mainline".to_string(),
            ));
        }

        Ok(())
    }

    pub fn deactivate(&self, _signer: Arc<dyn Signer>) -> Result<()> {
        println!("DidDht.deactivate() called");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::web5::crypto::dsa::ed25519::{self, Ed25519Generator, Ed25519Signer};

    use super::*;

    #[test]
    fn test_from_identity_key() {
        let private_jwk = Ed25519Generator::generate();
        let identity_key = ed25519::to_public_jwk(&private_jwk);
        let did_dht =
            DidDht::from_identity_key(identity_key.clone()).expect("Should create did:dht");

        assert_eq!(did_dht.did.method, "dht");
        assert_eq!(
            did_dht.document.verification_method[0].public_key_jwk,
            identity_key
        );
        assert_eq!(
            did_dht.document.verification_method[0].id,
            format!("{}#0", did_dht.did.uri)
        );
    }

    #[test]
    fn test_publish() {
        // Create did:dht
        let private_jwk = Ed25519Generator::generate();
        let identity_key = ed25519::to_public_jwk(&private_jwk);
        let did_dht =
            DidDht::from_identity_key(identity_key.clone()).expect("Should create did:dht");

        // Publish
        let signer = Ed25519Signer::new(private_jwk);
        did_dht
            .publish(Arc::new(signer))
            .expect("Should publish did");
    }

    #[test]
    fn test_resolve() {
        // Create did:dht
        let private_jwk = Ed25519Generator::generate();
        let identity_key = ed25519::to_public_jwk(&private_jwk);
        let did_dht =
            DidDht::from_identity_key(identity_key.clone()).expect("Should create did:dht");

        // Publish
        let signer = Ed25519Signer::new(private_jwk);
        did_dht
            .publish(Arc::new(signer))
            .expect("Should publish did");

        // Resolve from uri
        let resolved_did_dht = DidDht::resolve(&did_dht.did.uri);
        let resolved_document = resolved_did_dht.document.unwrap();
        assert_eq!(resolved_document, did_dht.document)
    }
}
