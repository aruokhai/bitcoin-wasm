use crate::signature::SignatureError;
use jwt_compact::{
    alg::Ed25519, jwk::JsonWebKey, jws::{
        alg::eddsa::EddsaJwsAlgorithm, serialize_compact, JwsAlgorithm, JwsHeader, JwsSigner,
        JwsVerifier,
    }, AlgorithmExt, Claim, Claims, Header, JoseError as JosekitError, Token
};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    sync::Arc,
};
use crate::{
    crypto::{
        dsa::{ed25519::Ed25519Verifier, DsaError, Signer as Web5Signer, Verifier as Web5Verifier},
        jwk::Jwk,
    },
    dids::data_model::document::Document,
};

#[derive(Clone)]
pub struct Signer {
    pub kid: String,
    pub web5_signer: Arc<dyn Web5Signer>,
}

impl JwsSigner for Signer {
    fn algorithm(&self) -> &dyn JwsAlgorithm {
        &EddsaJwsAlgorithm::Eddsa
    }

    fn key_id(&self) -> Option<&str> {
        Some(&self.kid)
    }

    fn signature_len(&self) -> usize {
        64
    }

    fn sign(&self, message: &[u8]) -> core::result::Result<Vec<u8>, JosekitError> {
        self.web5_signer
            .sign(message)
            // 🚧 improve error message semantics
            .map_err(|err| JosekitError::InvalidSignature(err.into()))
    }

    fn box_clone(&self) -> Box<dyn JwsSigner> {
        Box::new(self.clone())
    }
}

impl Debug for Signer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signer").field("kid", &self.kid).finish()
    }
}

impl Signer {
    pub fn sign_detached_compact_jws(&self, payload: &[u8]) -> Result<String, JosekitError> {
        let token =
        let compact_jws = serialize_compact(payload, &JwsHeader::new(), self)?;
        let parts: Vec<&str> = compact_jws.split('.').collect();
        let detached_compact_jws = format!("{}..{}", parts[0], parts[2]);
        Ok(detached_compact_jws)
    }

    pub fn sign_jwt(&self, payload: &Claims) -> Result<String, JosekitError> {
        let mut header = Header::empty();
        header.with_token_type("JWT");
        encode_with_signer(payload, &header, self)
    }
}

#[derive(Clone, Debug)]
pub struct Verifier {
    pub kid: String,
    pub public_jwk: Jwk,
}

impl JwsVerifier for Verifier {
    fn algorithm(&self) -> &dyn JwsAlgorithm {
        &EddsaJwsAlgorithm::Eddsa
    }

    fn key_id(&self) -> Option<&str> {
        Some(self.kid.as_str())
    }

    fn verify(&self, message: &[u8], signature: &[u8]) -> core::result::Result<(), JosekitError> {
        let verifier = Ed25519Verifier::new(self.public_jwk.clone());
        let result = verifier
            .verify(message, signature)
            .map_err(|e| JosekitError::InvalidSignature(e.into()))?;

        match result {
            true => Ok(()),
            false => Err(JosekitError::InvalidSignature(
                // 🚧 improve error message semantics
                DsaError::VerificationFailure("ed25519 verification failed".to_string()).into(),
            )),
        }
    }

    fn box_clone(&self) -> Box<dyn JwsVerifier> {
        Box::new(self.clone())
    }
}

fn create_selector<'a>(
    verifiers: &'a HashMap<String, Arc<Verifier>>,
) -> impl Fn(&JwsHeader) -> core::result::Result<Option<&'a dyn JwsVerifier>, JosekitError> + 'a {
    move |header: &JwsHeader| -> core::result::Result<Option<&'a dyn JwsVerifier>, JosekitError> {
        let kid = header.key_id().ok_or_else(|| {
            JosekitError::InvalidJwsFormat(SignatureError::Jose("missing kid".to_string()).into())
        })?;

        let verifier = verifiers.get(kid).ok_or_else(|| {
            JosekitError::InvalidJwsFormat(
                SignatureError::Jose("verification method not found".to_string()).into(),
            )
        })?;

        Ok(Some(&**verifier))
    }
}

impl Verifier {
    pub fn verify_compact_jws(document: Document, message: String) -> Result<(), JosekitError> {
        let verifiers: HashMap<String, Arc<Verifier>> = document
            .verification_method
            .iter()
            .map(|method| {
                (
                    method.id.clone(),
                    Arc::new(Verifier {
                        kid: method.id.clone(),
                        public_jwk: method.public_key_jwk.clone(),
                    }),
                )
            })
            .collect();

        let selector = create_selector(&verifiers);

        josekit::jws::deserialize_compact_with_selector(message, selector)?;

        Ok(())
    }
}
