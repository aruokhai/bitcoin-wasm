use super::{DsaError, Result, Signer, Verifier};
use crate::web5::crypto::jwk::Jwk;
use base64::{engine::general_purpose, Engine as _};
use ed25519_compact::{
    PublicKey, SecretKey, Seed, Signature
};




pub struct Ed25519Generator;

impl Ed25519Generator {
    pub fn generate() -> Jwk {
        let random_number = wasi::random::random::get_random_bytes(32);
        let mut seed = [0; 32];
        seed.copy_from_slice(random_number.as_slice());
        let seed = Seed::new(seed);
        let keypair = ed25519_compact::KeyPair::from_seed(seed);
        let verifying_key = keypair.pk;

        let binding = keypair.sk.to_vec();
        let private_key_bytes = binding.as_slice();
        let binding = verifying_key.to_vec();
        let public_key_bytes = binding.as_slice();

        Jwk {
            alg: Some("Ed25519".to_string()),
            kty: "OKP".to_string(),
            crv: "Ed25519".to_string(),
            x: general_purpose::URL_SAFE_NO_PAD.encode(public_key_bytes),
            d: Some(general_purpose::URL_SAFE_NO_PAD.encode(private_key_bytes)),
            ..Default::default()
        }
    }
}

pub(crate) fn public_jwk_from_bytes(public_key: &[u8]) -> Result<Jwk> {
    if public_key.len() != PublicKey::BYTES {
        return Err(DsaError::PublicKeyFailure(format!(
            "Public key has incorrect length {}",
            PublicKey::BYTES
        )));
    }

    Ok(Jwk {
        alg: Some("Ed25519".to_string()),
        kty: "OKP".to_string(),
        crv: "Ed25519".to_string(),
        x: general_purpose::URL_SAFE_NO_PAD.encode(public_key),
        ..Default::default()
    })
}

#[cfg(test)]
pub fn to_public_jwk(jwk: &Jwk) -> Jwk {
    Jwk {
        alg: jwk.alg.clone(),
        kty: jwk.kty.clone(),
        crv: jwk.crv.clone(),
        x: jwk.x.clone(),
        y: jwk.y.clone(),
        ..Default::default()
    }
}

pub(crate) fn public_jwk_extract_bytes(jwk: &Jwk) -> Result<Vec<u8>> {
    let decoded_x = general_purpose::URL_SAFE_NO_PAD.decode(&jwk.x)?;

    if decoded_x.len() != PublicKey::BYTES {
        return Err(DsaError::InvalidKeyLength(PublicKey::BYTES.to_string()));
    }

    Ok(decoded_x)
}

#[derive(Clone)]
pub struct Ed25519Signer {
    private_jwk: Jwk,
}

impl Ed25519Signer {
    pub fn new(private_jwk: Jwk) -> Self {
        Self { private_jwk }
    }
}

impl Signer for Ed25519Signer {
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let d = self
            .private_jwk
            .d
            .as_ref()
            .ok_or(DsaError::MissingPrivateKey)?;
        let decoded_d = general_purpose::URL_SAFE_NO_PAD.decode(d)?;
        if decoded_d.len() != SecretKey::BYTES {
            return Err(DsaError::InvalidKeyLength(SecretKey::BYTES.to_string()));
        }
        let mut key_array = [0u8; 64];
        key_array.copy_from_slice(&decoded_d);
        let signing_key = SecretKey::new(key_array);
        let signature = signing_key.sign(payload, None);
        Ok(signature.to_vec())
    }
    
    fn get_signing_key(&self) -> Result<SecretKey> {
        let d = self
            .private_jwk
            .d
            .as_ref()
            .ok_or(DsaError::MissingPrivateKey)?;
        let decoded_d = general_purpose::URL_SAFE_NO_PAD.decode(d)?;
        if decoded_d.len() != SecretKey::BYTES {
            return Err(DsaError::InvalidKeyLength(SecretKey::BYTES.to_string()));
        }
        let mut key_array = [0u8; 64];
        key_array.copy_from_slice(&decoded_d);
        let signing_key = SecretKey::new(key_array);
        return Ok(signing_key);
    }
}

#[derive(Clone)]
pub struct Ed25519Verifier {
    public_jwk: Jwk,
}

impl Ed25519Verifier {
    pub fn new(public_jwk: Jwk) -> Self {
        Self { public_jwk }
    }
}

impl Verifier for Ed25519Verifier {
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<bool> {
        let mut public_key_bytes = [0u8; PublicKey::BYTES];
        let decoded_x = general_purpose::URL_SAFE_NO_PAD.decode(&self.public_jwk.x)?;

        if decoded_x.len() != PublicKey::BYTES {
            return Err(DsaError::InvalidKeyLength(PublicKey::BYTES.to_string()));
        }

        public_key_bytes.copy_from_slice(&decoded_x);
        let verifying_key = PublicKey::new(public_key_bytes);

        if signature.len() != Signature::BYTES {
            return Err(DsaError::InvalidSignatureLength(self.public_jwk.x.clone()));
        }

        let mut signature_bytes = [0u8; Signature::BYTES];
        signature_bytes.copy_from_slice(signature);
        
        let verify_result = verifying_key.verify(payload, &Signature::from_slice(&signature_bytes).unwrap());
        match verify_result {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    fn get_verifying_key(&self) -> Result<PublicKey> {
        let mut public_key_bytes = [0u8; PublicKey::BYTES];
        let decoded_x = general_purpose::URL_SAFE_NO_PAD.decode(&self.public_jwk.x)?;

        if decoded_x.len() != PublicKey::BYTES {
            return Err(DsaError::InvalidKeyLength(PublicKey::BYTES.to_string()));
        }

        public_key_bytes.copy_from_slice(&decoded_x);
        let verifying_key = PublicKey::new(public_key_bytes);
        return Ok(verifying_key);
    }
}
