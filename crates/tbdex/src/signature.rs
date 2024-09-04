use base64::{engine::general_purpose, Engine};
use base64ct::{Base64UrlUnpadded, Encoding};
use ed25519_compact::{SecretKey, Signature};
use jwt_compact::alg::Ed25519;
use jwt_compact::{Algorithm, AlgorithmExt, AlgorithmSignature, Header};
use serde_json::Error as SerdeJsonError;
use serde_json::{Map, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use crate::web5::crypto::dsa::Verifier;
use crate::web5::dids::bearer_did::BearerDid;
use crate::web5::dids::data_model::document::Document;
use crate::web5::dids::{
    bearer_did::BearerDidError,
    resolution::{
        resolution_metadata::ResolutionMetadataError, resolution_result::ResolutionResult,
    },
};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum SignatureError {
    #[error("jose error {0}")]
    Jose(String),
    #[error(transparent)]
    ResolutionMetadata(#[from] ResolutionMetadataError),
    #[error(transparent)]
    BearerDid(#[from] BearerDidError),
    #[error("serde json error {0}")]
    SerdeJson(String),
    #[error("secretKeys")]
    SecretKey,
}



type Result<T> = std::result::Result<T, SignatureError>;

fn compute_digest(value: &Value) -> Result<Vec<u8>> {
    let canonical_string = serde_jcs::to_string(value).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(canonical_string.as_bytes());
    Ok(hasher.finalize().to_vec())
}

pub fn sign(bearer_did: &BearerDid, metadata: &Value, data: &Value) -> Result<String> {
    let mut combined = Map::new();
    combined.insert("metadata".to_string(), metadata.clone());
    combined.insert("data".to_string(), data.clone());

    let digest = compute_digest(&Value::Object(combined))?;

    // default to first VM
    let key_id = bearer_did.document.verification_method[0].id.clone();
    let web5_signer = bearer_did.get_signer(key_id.clone())?;
    let signing_key = web5_signer.get_signing_key()
        .map_err(|_| SignatureError::Jose("Cant get signing key".to_string()))?;
    let signature = serialise_compact(key_id, digest, signing_key);        
    Ok(signature)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompleteHeader {
    #[serde(rename = "alg")]
    pub algorithm: String ,
    #[serde(rename = "kid")]
    pub kid: String,


}

pub fn serialise_compact(kid: String, digest: Vec<u8>, signing_key: SecretKey) -> String {
    let complete_header = CompleteHeader{ kid, algorithm: Ed25519.name().to_string()};
    let header = serde_json::to_string(&complete_header).unwrap();
    let mut buffer = Vec::new();
    encode_base64_buf(&header, &mut buffer);
    buffer.push(b'.');
    encode_base64_buf(&digest, &mut buffer);
    let signature =  Ed25519.sign(&signing_key, &buffer);
    buffer.push(b'.');
    encode_base64_buf(signature.as_bytes(), &mut buffer);
    let compact_jws =  unsafe { String::from_utf8_unchecked(buffer) };
    let parts: Vec<&str> = compact_jws.split('.').collect();
    format!("{}..{}", parts[0], parts[2])

}

fn encode_base64_buf(source: impl AsRef<[u8]>, buffer: &mut Vec<u8>) {
    let source = source.as_ref();
    let previous_len = buffer.len();
    let claims_len = Base64UrlUnpadded::encoded_len(source);
    buffer.resize(previous_len + claims_len, 0);
    Base64UrlUnpadded::encode(source, &mut buffer[previous_len..])
        .expect("miscalculated base64-encoded length; this should never happen");
}


// pub fn verify(
//     did_uri: &str,
//     metadata: &Value,
//     data: &Value,
//     detached_compact_jws: &str,
// ) -> Result<()> {
//     // re-attach the payload
//     let mut combined = Map::new();
//     combined.insert("metadata".to_string(), metadata.clone());
//     combined.insert("data".to_string(), data.clone());
//     let digest = compute_digest(&Value::Object(combined))?;
//     let payload = general_purpose::URL_SAFE_NO_PAD.encode(digest);

//     let parts: Vec<&str> = detached_compact_jws.split('.').collect();
//     if parts.len() != 3 {
//         return Err(SignatureError::Jose(
//             "detached compact jws wrong number of parts".to_string(),
//         ));
//     }
//     let message = format!("{}.{}.{}", parts[0], payload, parts[2]);

//     let resolution_result = ResolutionResult::new(did_uri);
//     match resolution_result.resolution_metadata.error {
//         Some(e) => Err(SignatureError::ResolutionMetadata(e)),
//         None => {
//             let document = resolution_result
//                 .document
//                 .ok_or(SignatureError::ResolutionMetadata(
//                     ResolutionMetadataError::InternalError,
//                 ))?;

//             Ed25519.verify_signature(signature, verifying_key, message)::verify_compact_jws(document, message)?;

//             Ok(())
//         }
//     }
// }

// pub fn verify_compact_jws(document: Document, message: String) -> Result<(), JosekitError> {
//     let verifiers: HashMap<String, Arc<Verifier>> = document
//         .verification_method
//         .iter()
//         .map(|method| {
//             (
//                 method.id.clone(),
//                 Arc::new(Verifier {
//                     kid: method.id.clone(),
//                     public_jwk: method.public_key_jwk.clone(),
//                 }),
//             )
//         })
//         .collect();

//     let selector = create_selector(&verifiers);

//     josekit::jws::deserialize_compact_with_selector(message, selector)?;

//     Ok(())
// }