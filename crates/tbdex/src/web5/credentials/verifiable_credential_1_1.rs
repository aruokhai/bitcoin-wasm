use super::{CredentialError, Result};
use crate::web5::{
    crypto::dsa::{ed25519::Ed25519Verifier, DsaError, Signer, Verifier},
    dids::{
        bearer_did::BearerDid,
        did::Did,
        resolution::{
            resolution_metadata::ResolutionMetadataError, resolution_result::ResolutionResult,
        },
    },
};
use chrono::{DateTime, Utc};
use wasi::clocks::wall_clock;
use core::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Arc,
    time::{UNIX_EPOCH},
};
use wasi::clocks::wall_clock::Datetime;
use jwt_compact::{alg::{Ed25519, SigningKey}, prelude::*};


pub const BASE_CONTEXT: &str = "https://www.w3.org/2018/credentials/v1";
pub const BASE_TYPE: &str = "VerifiableCredential";

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct NamedIssuer {
    pub id: String,
    pub name: String,
}


#[derive(Debug, Serialize, Deserialize)]
struct CustomClaims {
    /// `sub` is a standard claim which denotes claim subject:
    /// https://tools.ietf.org/html/rfc7519#section-4.1.2
    #[serde(rename = "iss")]
    issuer: String,
    #[serde(rename = "ndf", )]
    not_before: u64,
    #[serde(rename = "sub")]
    subject: String,
    #[serde(rename = "exp" ,
    skip_serializing_if = "Option::is_none")]
    expiration_time: Option<u64>,
    #[serde(rename = "jti")]
    jwt_id: String,
    #[serde(rename = "vc")]
    validated_credential: serde_json::Value,
    #[serde(rename = "iat" , )]
    issued_at: u64

}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Issuer {
    String(String),
    Object(NamedIssuer),
}

impl<I> From<I> for Issuer
where
    I: Into<String>,
{
    fn from(s: I) -> Self {
        Issuer::String(s.into())
    }
}

impl Display for Issuer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Issuer::String(s) => write!(f, "{}", s),
            Issuer::Object(ni) => write!(f, "{}", ni.id),
        }
    }
}

fn serialize_system_time<S>(
    time: &wall_clock::Datetime,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let datetime = chrono::DateTime::from_timestamp(time.seconds as i64, time.nanoseconds).ok_or(serde::ser::Error::custom("error converting timestamp to DateTime"))?;
    let s = datetime.to_rfc3339();
    serializer.serialize_str(&s)
}

fn deserialize_system_time<'de, D>(deserializer: D) -> std::result::Result<wall_clock::Datetime, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let datetime = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
    let timestamp =  datetime.timestamp();
    let system_time = wall_clock::Datetime{seconds: timestamp as u64, nanoseconds: 0};
    Ok(system_time)
}

fn serialize_option_system_time<S>(
    time: &Option<wall_clock::Datetime>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match time {
        Some(time) => serialize_system_time(time, serializer),
        None => serializer.serialize_none(),
    }
}

fn deserialize_option_system_time<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<wall_clock::Datetime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let datetime = chrono::DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
            let timestamp =  datetime.timestamp();
            let system_time = wall_clock::Datetime{seconds: timestamp as u64, nanoseconds: 0};
            Ok(Some(system_time))
        }
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: Vec<String>,
    pub issuer: Issuer,
    #[serde(
        rename = "issuanceDate",
        serialize_with = "serialize_system_time",
        deserialize_with = "deserialize_system_time"
    )]
    pub issuance_date: wall_clock::Datetime,
    #[serde(
        rename = "expirationDate",
        serialize_with = "serialize_option_system_time",
        deserialize_with = "deserialize_option_system_time"
    )]
    pub expiration_date: Option<wall_clock::Datetime>,
    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CredentialSubject {
    pub id: String,
    #[serde(flatten)]
    pub params: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtPayloadVerifiableCredential {
    #[serde(rename = "@context")]
    context: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(rename = "type")]
    r#type: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<Issuer>,
    #[serde(
        rename = "issuanceDate",
        serialize_with = "serialize_option_system_time",
        deserialize_with = "deserialize_option_system_time"
    )]
    issuance_date: Option<wall_clock::Datetime>,
    #[serde(
        rename = "expirationDate",
        serialize_with = "serialize_option_system_time",
        deserialize_with = "deserialize_option_system_time"
    )]
    expiration_date: Option<wall_clock::Datetime>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "credentialSubject")]
    credential_subject: Option<CredentialSubject>,
}

impl VerifiableCredential {
    pub fn new(
        id: String,
        context: Vec<String>,
        r#type: Vec<String>,
        issuer: Issuer,
        issuance_date: wall_clock::Datetime,
        expiration_date: Option<wall_clock::Datetime>,
        credential_subject: CredentialSubject,
    ) -> Self {
        let context_with_base = std::iter::once(BASE_CONTEXT.to_string())
            .chain(context.into_iter().filter(|c| c != BASE_CONTEXT))
            .collect::<Vec<_>>();

        let type_with_base = std::iter::once(BASE_TYPE.to_string())
            .chain(r#type.into_iter().filter(|t| t != BASE_TYPE))
            .collect::<Vec<_>>();

        Self {
            context: context_with_base,
            id,
            r#type: type_with_base,
            issuer,
            issuance_date,
            expiration_date,
            credential_subject,
        }
    }

    pub fn sign(&self, bearer_did: &BearerDid) -> Result<String> {
        // default to first VM
        let key_id = bearer_did.document.verification_method[0].id.clone();
        let signer = bearer_did.get_signer(key_id.clone())?;

        self.sign_with_signer(&key_id, signer)
    }

    pub fn sign_with_signer(&self, key_id: &str, signer: Arc<dyn Signer>) -> Result<String> {
        let vc_claim = JwtPayloadVerifiableCredential {
            context: self.context.clone(),
            id: Some(self.id.clone()),
            r#type: self.r#type.clone(),
            issuer: Some(self.issuer.clone()),
            issuance_date: Some(self.issuance_date),
            expiration_date: self.expiration_date,
            credential_subject: Some(self.credential_subject.clone()),
        };
        let mut expiration_time = None;
        if let Some(exp) = self.expiration_date.clone() {
            expiration_time = Some(exp.seconds)
        }
        let  payload = Claims::new(CustomClaims{
            issuer: self.issuer.to_string(),
            jwt_id: self.credential_subject.id.clone(),
            validated_credential: serde_json::to_value(vc_claim)?,
            subject: self.credential_subject.id.clone(),
            not_before: self.issuance_date.seconds,
            issued_at: wall_clock::now().seconds,
            expiration_time
        });
        let  header = Header::empty()
        .with_token_type("JWT")
        .with_key_id(key_id);
        let vc_jwt = Ed25519.token(&header, &payload, &signer.get_signing_key().unwrap()).unwrap();
        
        Ok(vc_jwt)
    }

    pub fn verify(vc_jwt: &str) -> Result<Self> {
        // this function currently only supports Ed25519
        let token_result = UntrustedToken::new(vc_jwt);
        let token = token_result.unwrap();

        let kid = token.header().clone()
            .key_id
            .unwrap();

        let did = Did::new(&kid)?;

        let resolution_result = ResolutionResult::new(&did.uri);
        if let Some(err) = resolution_result.resolution_metadata.error.clone() {
            return Err(CredentialError::Resolution(err));
        }

        let public_key_jwk = resolution_result
            .document
            .unwrap()
            .find_public_key_jwk(kid.to_string())?;

        let verifier = Ed25519Verifier::new(public_key_jwk);

        Self::verify_with_verifier(vc_jwt, Arc::new(verifier))
    }

    pub fn verify_with_verifier(vc_jwt: &str, verifier: Arc<dyn Verifier>) -> Result<Self> {
        let token: UntrustedToken = vc_jwt.try_into().unwrap();

        let kid = <std::option::Option<std::string::String> as Clone>::clone(&token.header()
            .key_id)
            .unwrap();

        let signed_token = Ed25519.validator::<'static,CustomClaims>(&verifier.get_verifying_key().unwrap()).validate_for_signed_token(&token).unwrap();
        let jwt_payload = signed_token.token.claims();

        let vc_claim = &jwt_payload
            .custom
            .validated_credential;
        let vc_payload =
            serde_json::from_value::<JwtPayloadVerifiableCredential>(vc_claim.clone())?;

        // registered claims checks
        let jti = &jwt_payload
            .custom
            .jwt_id;
        let iss = &jwt_payload
            .custom
            .issuer;
        let sub = &jwt_payload
            .custom
            .subject;
        let nbf = jwt_payload
            .custom
            .not_before;
        let exp = jwt_payload.custom.expiration_time;

        if let Some(id) = &vc_payload.id {
            if id != jti {
                return Err(CredentialError::ClaimMismatch("id".to_string()));
            }
        }

        if let Some(issuer) = &vc_payload.issuer {
            let vc_issuer = issuer.to_string();
            if *iss != vc_issuer {
                return Err(CredentialError::ClaimMismatch("issuer".to_string()));
            }
        }

        if let Some(credential_subject) = &vc_payload.credential_subject {
            if *sub != credential_subject.id {
                return Err(CredentialError::ClaimMismatch("subject".to_string()));
            }
        }

        let now = wall_clock::now();
        match vc_payload.expiration_date {
            Some(ref vc_payload_expiration_date) => match exp {
                None => {
                    return Err(CredentialError::MisconfiguredExpirationDate(
                        "VC has expiration date but no exp in registered claims".to_string(),
                    ));
                }
                Some(exp) => {
                    if vc_payload_expiration_date
                        .seconds
                        != exp
                    {
                        return Err(CredentialError::ClaimMismatch(
                            "expiration_date".to_string(),
                        ));
                    }

                    if now.seconds > exp {
                        return Err(CredentialError::CredentialExpired);
                    }
                }
            },
            None => {
                if let Some(exp) = exp {
                    if now.seconds  > exp {
                        return Err(CredentialError::CredentialExpired);
                    }
                }
            }
        }

        let vc_issuer = vc_payload.issuer.unwrap_or(Issuer::String(iss.to_string()));

        let vc_credential_subject = vc_payload.credential_subject.unwrap_or(CredentialSubject {
            id: sub.to_string(),
            params: None,
        });

        let vc = VerifiableCredential {
            context: vc_payload.context,
            id: jti.to_string(),
            r#type: vc_payload.r#type,
            issuer: vc_issuer,
            issuance_date: Datetime{ seconds: nbf.clone(), nanoseconds: 0 },
            // TODO: Fix the date time
            expiration_date: None,
            credential_subject: vc_credential_subject,
        };

        validate_vc_data_model(&vc)?;

        Ok(vc)
    }
}

fn validate_vc_data_model(vc: &VerifiableCredential) -> Result<()> {
    // Required fields ["@context", "id", "type", "issuer", "issuanceDate", "credentialSubject"]
    if vc.id.is_empty() {
        return Err(CredentialError::VcDataModelValidationError(
            "missing id".to_string(),
        ));
    }

    if vc.context.is_empty() || vc.context[0] != BASE_CONTEXT {
        return Err(CredentialError::VcDataModelValidationError(
            "missing context".to_string(),
        ));
    }

    if vc.r#type.is_empty() || vc.r#type[0] != BASE_TYPE {
        return Err(CredentialError::VcDataModelValidationError(
            "missing type".to_string(),
        ));
    }

    if vc.issuer.to_string().is_empty() {
        return Err(CredentialError::VcDataModelValidationError(
            "missing issuer".to_string(),
        ));
    }

    if vc.credential_subject.id.is_empty() {
        return Err(CredentialError::VcDataModelValidationError(
            "missing credential subject".to_string(),
        ));
    }

    let now = wall_clock::now();
    if vc.issuance_date.seconds > now.seconds {
        return Err(CredentialError::VcDataModelValidationError(
            "issuance date in future".to_string(),
        ));
    }

    // Validate expiration date if it exists
    if let Some(expiration_date) = &vc.expiration_date {
        if expiration_date.seconds < now.seconds {
            return Err(CredentialError::VcDataModelValidationError(
                "credential expired".to_string(),
            ));
        }
    }

    // TODO: Add validations to credential_status, credential_schema, and evidence once they are added to the VcDataModel
    // https://github.com/TBD54566975/web5-rs/issues/112

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         crypto::{
//             dsa::ed25519::Ed25519Generator, key_managers::in_memory_key_manager::InMemoryKeyManager,
//         },
//         dids::methods::did_jwk::DidJwk,
//     };
//     use std::time::Duration;
//     use uuid::Uuid;

//     #[test]
//     fn can_create_sign_and_verify() {
//         let key_manager = InMemoryKeyManager::new();
//         let public_jwk = key_manager
//             .import_private_jwk(Ed25519Generator::generate())
//             .unwrap();
//         let did_jwk = DidJwk::from_public_jwk(public_jwk).unwrap();
//         let bearer_did = BearerDid::new(&did_jwk.did.uri, Arc::new(key_manager)).unwrap();

//         let now = wall_clock::Datetime::now();
//         let vc = VerifiableCredential::new(
//             format!("urn:vc:uuid:{0}", Uuid::new_v4().to_string()),
//             vec![BASE_CONTEXT.to_string()],
//             vec![BASE_TYPE.to_string()],
//             Issuer::String(bearer_did.did.uri.clone()),
//             now,
//             Some(now + Duration::from_secs(20 * 365 * 24 * 60 * 60)), // now + 20 years
//             CredentialSubject {
//                 id: bearer_did.did.uri.clone(),
//                 ..Default::default()
//             },
//         );

//         let vc_jwt = vc.sign(&bearer_did).unwrap();
//         assert_ne!(String::default(), vc_jwt);

//         VerifiableCredential::verify(&vc_jwt).unwrap();
//     }
// }
