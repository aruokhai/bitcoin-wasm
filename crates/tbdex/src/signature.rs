use crate::jose::{Signer, Verifier};
use base64::{engine::general_purpose, Engine};
use josekit::JoseError as JosekitError;
use serde_json::Error as SerdeJsonError;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fmt::Debug;
use web5::dids::bearer_did::BearerDid;
use web5::dids::{
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
}

impl From<SerdeJsonError> for SignatureError {
    fn from(err: SerdeJsonError) -> Self {
        SignatureError::SerdeJson(err.to_string())
    }
}

impl From<JosekitError> for SignatureError {
    fn from(err: JosekitError) -> Self {
        SignatureError::Jose(err.to_string())
    }
}

type Result<T> = std::result::Result<T, SignatureError>;

fn compute_digest(value: &Value) -> Result<Vec<u8>> {
    let canonical_string = serde_jcs::to_string(value)?;
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
    let jose_signer = Signer {
        kid: key_id,
        web5_signer,
    };
    let detached_compact_jws = jose_signer.sign_detached_compact_jws(&digest)?;

    Ok(detached_compact_jws)
}

pub fn verify(
    did_uri: &str,
    metadata: &Value,
    data: &Value,
    detached_compact_jws: &str,
) -> Result<()> {
    // re-attach the payload
    let mut combined = Map::new();
    combined.insert("metadata".to_string(), metadata.clone());
    combined.insert("data".to_string(), data.clone());
    let digest = compute_digest(&Value::Object(combined))?;
    let payload = general_purpose::URL_SAFE_NO_PAD.encode(digest);

    let parts: Vec<&str> = detached_compact_jws.split('.').collect();
    if parts.len() != 3 {
        return Err(SignatureError::Jose(
            "detached compact jws wrong number of parts".to_string(),
        ));
    }
    let message = format!("{}.{}.{}", parts[0], payload, parts[2]);

    let resolution_result = ResolutionResult::new(did_uri);
    match resolution_result.resolution_metadata.error {
        Some(e) => Err(SignatureError::ResolutionMetadata(e)),
        None => {
            let document = resolution_result
                .document
                .ok_or(SignatureError::ResolutionMetadata(
                    ResolutionMetadataError::InternalError,
                ))?;

            Verifier::verify_compact_jws(document, message)?;

            Ok(())
        }
    }
}
