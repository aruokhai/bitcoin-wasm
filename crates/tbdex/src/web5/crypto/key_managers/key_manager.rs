use super::Result;
use crate::web5::crypto::{dsa::Signer, jwk::Jwk};
use std::sync::Arc;

pub trait KeyManager: Send + Sync {
    fn get_signer(&self, public_jwk: Jwk) -> Result<Arc<dyn Signer>>;
}
