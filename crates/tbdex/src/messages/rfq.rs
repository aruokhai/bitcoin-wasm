use super::{MessageKind, MessageMetadata, Result};
use crate::{
    json::{FromJson, ToJson},
    json_schemas::generated::{
        MESSAGE_JSON_SCHEMA, RFQ_DATA_JSON_SCHEMA, RFQ_PRIVATE_DATA_JSON_SCHEMA,
    },
    messages::MessageError,
    resources::offering::Offering,
    DEFAULT_PROTOCOL_VERSION,
};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use web5::{
    credentials::verifiable_credential_1_1::VerifiableCredential, dids::bearer_did::BearerDid,
};

#[derive(Clone, Serialize, Default, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rfq {
    pub metadata: MessageMetadata,
    pub data: RfqData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_data: Option<RfqPrivateData>,
    pub signature: String,
}

impl ToJson for Rfq {}
impl FromJson for Rfq {}

impl Rfq {
    pub fn create(
        to: &str,
        from: &str,
        create_rfq_data: &CreateRfqData,
        protocol: Option<String>,
        external_id: Option<String>,
    ) -> Result<Self> {
        let id = MessageKind::Rfq.typesafe_id()?;

        let metadata = MessageMetadata {
            from: from.to_string(),
            to: to.to_string(),
            kind: MessageKind::Rfq,
            id: id.clone(),
            exchange_id: id.clone(),
            external_id,
            protocol: protocol.unwrap_or_else(|| DEFAULT_PROTOCOL_VERSION.to_string()),
            created_at: Utc::now().to_rfc3339(),
        };

        let (data, private_data) = hash_private_data(create_rfq_data)?;

        let rfq = Self {
            metadata: metadata.clone(),
            data: data.clone(),
            private_data: Some(private_data),
            signature: String::default(),
        };

        Ok(rfq)
    }

    pub fn sign(&mut self, bearer_did: &BearerDid) -> Result<()> {
        self.signature = crate::signature::sign(
            bearer_did,
            &serde_json::to_value(&self.metadata)?,
            &serde_json::to_value(&self.data)?,
        )?;
        Ok(())
    }

    pub fn verify(&self) -> Result<()> {
        // verify resource json schema
        crate::json_schemas::validate_from_str(MESSAGE_JSON_SCHEMA, self)?;

        // verify data json schema
        crate::json_schemas::validate_from_str(RFQ_DATA_JSON_SCHEMA, &self.data)?;

        // verify private data json schema
        if let Some(private_data) = &self.private_data {
            crate::json_schemas::validate_from_str(RFQ_PRIVATE_DATA_JSON_SCHEMA, private_data)?;
        }

        // verify signature
        crate::signature::verify(
            &self.metadata.from,
            &serde_json::to_value(self.metadata.clone())?,
            &serde_json::to_value(self.data.clone())?,
            &self.signature,
        )?;

        Ok(())
    }

    pub fn verify_offering_requirements(&self, offering: &Offering) -> Result<()> {
        // verify protocol version
        if offering.metadata.protocol != self.metadata.protocol {
            return Err(MessageError::OfferingVerification(format!(
                "offering has protocol version {} but rfq has protocol version {}",
                offering.metadata.protocol, self.metadata.protocol
            )));
        }

        // verify offering id
        if offering.metadata.id != self.data.offering_id {
            return Err(MessageError::OfferingVerification(format!(
                "offering id is {} but rfq has offering id {}",
                offering.metadata.id, self.data.offering_id
            )));
        }

        let payin_amount = self.data.payin.amount.parse::<f64>().map_err(|_| {
            MessageError::OfferingVerification(format!(
                "rfq payin amount invalid decimal string {}",
                self.data.payin.amount
            ))
        })?;

        // verify max amount
        if let Some(max_amount) = offering.data.payin.max.as_ref() {
            let max_amount = max_amount.parse::<f64>().map_err(|_| {
                MessageError::OfferingVerification(format!(
                    "offering max amount invalid decimal string {}",
                    max_amount
                ))
            })?;

            if payin_amount > max_amount {
                return Err(MessageError::OfferingVerification(format!(
                    "rfq payin of {} is larger than max offering amount of {}",
                    payin_amount, max_amount
                )));
            }
        }

        // verify min amount
        if let Some(min_amount) = offering.data.payin.min.as_ref() {
            let min_amount = min_amount.parse::<f64>().map_err(|_| {
                MessageError::OfferingVerification(format!(
                    "offering min amount invalid decimal string {}",
                    min_amount
                ))
            })?;

            if payin_amount < min_amount {
                return Err(MessageError::OfferingVerification(format!(
                    "rfq payin of {} is smaller than min offering amount of {}",
                    payin_amount, min_amount
                )));
            }
        }

        let private_data = match &self.private_data {
            None => {
                return Err(MessageError::OfferingVerification(
                    "cannot verify offering requirements without private data".to_string(),
                ))
            }
            Some(pd) => pd,
        };

        // verify payin json schema
        if let Some(payin_method) = offering
            .data
            .payin
            .methods
            .iter()
            .find(|m| m.kind == self.data.payin.kind)
        {
            if let Some(json_schema) = &payin_method.required_payment_details {
                let payment_details = private_data
                    .payin
                    .as_ref()
                    .ok_or_else(|| {
                        MessageError::OfferingVerification("missing private payin data".to_string())
                    })?
                    .payment_details
                    .as_ref()
                    .ok_or_else(|| {
                        MessageError::OfferingVerification("missing payment details".to_string())
                    })?;

                crate::json_schemas::validate(json_schema, payment_details)?;
            }
        } else {
            return Err(MessageError::OfferingVerification(format!(
                "kind {} not found in offering",
                self.data.payin.kind
            )));
        }

        // verify payout json schema
        if let Some(payout_method) = offering
            .data
            .payout
            .methods
            .iter()
            .find(|m| m.kind == self.data.payout.kind)
        {
            if let Some(json_schema) = &payout_method.required_payment_details {
                let payment_details = private_data
                    .payout
                    .as_ref()
                    .ok_or_else(|| {
                        MessageError::OfferingVerification(
                            "missing private payout data".to_string(),
                        )
                    })?
                    .payment_details
                    .as_ref()
                    .ok_or_else(|| {
                        MessageError::OfferingVerification("missing payment details".to_string())
                    })?;

                crate::json_schemas::validate(json_schema, payment_details)?;
            }
        } else {
            return Err(MessageError::OfferingVerification(format!(
                "kind {} not found in offering",
                self.data.payout.kind
            )));
        }

        // verify claims
        if let Some(required_claims) = &offering.data.required_claims {
            let vc_jwts = required_claims
                .select_credentials(&private_data.claims.clone().unwrap_or_default())
                .map_err(|_| {
                    MessageError::OfferingVerification("failed to select credentials".to_string())
                })?;

            if vc_jwts.is_empty() {
                return Err(MessageError::OfferingVerification(
                    "no matching credentials found".to_string(),
                ));
            }

            for vc_jwt in vc_jwts {
                VerifiableCredential::verify(&vc_jwt).map_err(|_| {
                    MessageError::OfferingVerification(format!(
                        "vc_jwt failed verifiction {}",
                        vc_jwt
                    ))
                })?;
            }
        }

        Ok(())
    }

    pub fn verify_all_private_data(&self) -> Result<()> {
        let private_data = match &self.private_data {
            None => {
                return Err(MessageError::PrivateDataVerification(
                    "cannot verify all private data without private data".to_string(),
                ))
            }
            Some(pd) => pd,
        };

        if let Some(hash) = &self.data.payin.payment_details_hash {
            if let Some(payin) = &private_data.payin {
                let digest = digest_private_data(&private_data.salt, &payin.payment_details)?;
                if &digest != hash {
                    return Err(MessageError::PrivateDataVerification(
                        "private data payin hash mismatch".to_string(),
                    ));
                }
            } else {
                return Err(MessageError::PrivateDataVerification(
                    "private data missing payin".to_string(),
                ));
            }
        }

        if let Some(hash) = &self.data.payout.payment_details_hash {
            if let Some(payout) = &private_data.payout {
                let digest = digest_private_data(&private_data.salt, &payout.payment_details)?;
                if &digest != hash {
                    return Err(MessageError::PrivateDataVerification(
                        "private data payout hash mismatch".to_string(),
                    ));
                }
            } else {
                return Err(MessageError::PrivateDataVerification(
                    "private data missing payout".to_string(),
                ));
            }
        }

        if let Some(hash) = &self.data.claims_hash {
            let digest = digest_private_data(&private_data.salt, &private_data.claims)?;
            if &digest != hash {
                return Err(MessageError::PrivateDataVerification(
                    "private data claims hash mismatch".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn verify_present_private_data(&self) -> Result<()> {
        if let Some(private_data) = &self.private_data {
            let salt = match &self.private_data {
                None => {
                    if private_data.payin.is_some()
                        || private_data.payout.is_some()
                        || private_data.claims.is_some()
                    {
                        return Err(MessageError::PrivateDataVerification(
                            "private data defined but salt is not defined".to_string(),
                        ));
                    } else {
                        return Ok(());
                    }
                }
                Some(pd) => pd.salt.clone(),
            };

            if let Some(payin) = &private_data.payin {
                if let Some(hash) = &self.data.payin.payment_details_hash {
                    let digest = digest_private_data(&salt, &payin.payment_details)?;
                    if &digest != hash {
                        return Err(MessageError::PrivateDataVerification(
                            "private data payin hash mismatch".to_string(),
                        ));
                    }
                } else {
                    return Err(MessageError::PrivateDataVerification(
                        "private data payin defined but hash is not defined".to_string(),
                    ));
                }
            }

            if let Some(payout) = &private_data.payout {
                if let Some(hash) = &self.data.payout.payment_details_hash {
                    let digest = digest_private_data(&salt, &payout.payment_details)?;
                    if &digest != hash {
                        return Err(MessageError::PrivateDataVerification(
                            "private data payout hash mismatch".to_string(),
                        ));
                    }
                } else {
                    return Err(MessageError::PrivateDataVerification(
                        "private data payout defined but hash is not defined".to_string(),
                    ));
                }
            }

            if let Some(claims) = &private_data.claims {
                if let Some(hash) = &self.data.claims_hash {
                    let digest = digest_private_data(&salt, &claims)?;
                    if &digest != hash {
                        return Err(MessageError::PrivateDataVerification(
                            "private data claims hash mismatch".to_string(),
                        ));
                    }
                } else {
                    return Err(MessageError::PrivateDataVerification(
                        "private data claims defined but hash is not defined".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRfqData {
    pub offering_id: String,
    pub payin: CreateSelectedPayinMethod,
    pub payout: CreateSelectedPayoutMethod,
    pub claims: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSelectedPayinMethod {
    pub kind: String,
    pub payment_details: Option<serde_json::Value>,
    pub amount: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSelectedPayoutMethod {
    pub kind: String,
    pub payment_details: Option<serde_json::Value>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RfqData {
    pub offering_id: String,
    pub payin: SelectedPayinMethod,
    pub payout: SelectedPayoutMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims_hash: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedPayinMethod {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details_hash: Option<String>,
    pub amount: String,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SelectedPayoutMethod {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details_hash: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RfqPrivateData {
    pub salt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payin: Option<PrivatePaymentDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout: Option<PrivatePaymentDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<String>>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrivatePaymentDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_details: Option<serde_json::Value>,
}

fn hash_private_data(create_rfq_data: &CreateRfqData) -> Result<(RfqData, RfqPrivateData)> {
    let salt = generate_random_salt();

    let payin_payment_details_hash = create_rfq_data
        .payin
        .payment_details
        .as_ref()
        .map(|pd| digest_private_data(&salt, pd))
        .transpose()?;
    let payout_payment_details_hash = create_rfq_data
        .payout
        .payment_details
        .as_ref()
        .map(|pd| digest_private_data(&salt, pd))
        .transpose()?;
    let claims_hash = if create_rfq_data.claims.is_empty() {
        None
    } else {
        Some(digest_private_data(&salt, &create_rfq_data.claims)?)
    };

    let hashed_rfq_data = RfqData {
        offering_id: create_rfq_data.offering_id.clone(),
        payin: SelectedPayinMethod {
            kind: create_rfq_data.payin.kind.clone(),
            payment_details_hash: payin_payment_details_hash,
            amount: create_rfq_data.payin.amount.clone(),
        },
        payout: SelectedPayoutMethod {
            kind: create_rfq_data.payout.kind.clone(),
            payment_details_hash: payout_payment_details_hash,
        },
        claims_hash,
    };

    let private_rfq_data =
        RfqPrivateData {
            salt: salt.clone(),
            payin: create_rfq_data
                .payin
                .payment_details
                .as_ref()
                .map(|pd| PrivatePaymentDetails {
                    payment_details: Some(pd.clone()),
                }),
            payout: create_rfq_data.payout.payment_details.as_ref().map(|pd| {
                PrivatePaymentDetails {
                    payment_details: Some(pd.clone()),
                }
            }),
            claims: if !create_rfq_data.claims.is_empty() {
                Some(create_rfq_data.claims.clone())
            } else {
                None
            },
        };

    Ok((hashed_rfq_data, private_rfq_data))
}

fn generate_random_salt() -> String {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    general_purpose::URL_SAFE_NO_PAD.encode(salt)
}

fn digest_private_data<T: Serialize>(salt: &str, value: &T) -> Result<String> {
    let digestible = serde_json::json!([salt, value]);
    let serialized = serde_jcs::to_string(&digestible)?;

    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let digest = hasher.finalize();

    Ok(general_purpose::URL_SAFE_NO_PAD.encode(digest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use web5::{
        crypto::{
            dsa::ed25519::Ed25519Generator, key_managers::in_memory_key_manager::InMemoryKeyManager,
        },
        dids::methods::did_jwk::DidJwk,
    };

    #[test]
    fn can_create_and_sign() {
        let key_manager = InMemoryKeyManager::new();
        let public_jwk = key_manager
            .import_private_jwk(Ed25519Generator::generate())
            .unwrap();
        let did_jwk = DidJwk::from_public_jwk(public_jwk).unwrap();

        let bearer_did = BearerDid::new(&did_jwk.did.uri, Arc::new(key_manager)).unwrap();

        let mut rfq = Rfq::create(
            "did:test:pfi",
            &did_jwk.did.uri,
            &CreateRfqData {
                offering_id: "offering_123".to_string(),
                payin: CreateSelectedPayinMethod {
                    kind: "BTC".to_string(),
                    payment_details: Some(serde_json::json!({"tmp": "payment-details"})),
                    amount: "101".to_string(),
                },
                payout: CreateSelectedPayoutMethod {
                    kind: "BTC".to_string(),
                    payment_details: Some(serde_json::json!({"tmp": "payment-details"})),
                },
                claims: vec!["some-claim".to_string()],
            },
            None,
            None,
        )
        .unwrap();

        rfq.sign(&bearer_did).unwrap();

        assert_ne!(String::default(), rfq.signature);

        let rfq_json_string = rfq.to_json_string().unwrap();

        assert_ne!(String::default(), rfq_json_string);

        let parsed_rfq = Rfq::from_json_string(&rfq_json_string).unwrap();

        assert_eq!(rfq, parsed_rfq);
    }
}

#[cfg(test)]
mod tbdex_test_vectors_protocol {
    use super::*;
    use std::fs;

    #[derive(Debug, serde::Deserialize)]
    pub struct TestVector {
        pub input: String,
        pub output: Rfq,
    }

    #[test]
    fn parse_rfq() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-rfq.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector: TestVector = serde_json::from_str(&test_vector_json).unwrap();
        let parsed_rfq: Rfq = Rfq::from_json_string(&test_vector.input).unwrap();

        parsed_rfq.verify_all_private_data().unwrap();

        assert_eq!(test_vector.output, parsed_rfq);
    }

    #[test]
    fn parse_rfq_omit_private_data() {
        let path =
            "../../tbdex/hosted/test-vectors/protocol/vectors/parse-rfq-omit-private-data.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector: TestVector = serde_json::from_str(&test_vector_json).unwrap();
        let parsed_rfq: Rfq = Rfq::from_json_string(&test_vector.input).unwrap();

        parsed_rfq.verify_present_private_data().unwrap();

        assert_eq!(test_vector.output, parsed_rfq);
    }
}
