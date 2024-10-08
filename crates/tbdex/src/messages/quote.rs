use super::{MessageKind, MessageMetadata, Result};
use crate::{
    get_utc_now, json::{FromJson, ToJson}, json_schemas::generated::{MESSAGE_JSON_SCHEMA, QUOTE_DATA_JSON_SCHEMA}, DEFAULT_PROTOCOL_VERSION
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::web5::dids::bearer_did::BearerDid;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Quote {
    pub metadata: MessageMetadata,
    pub data: QuoteData,
    pub signature: String,
}

impl ToJson for Quote {}
impl FromJson for Quote {}

impl Quote {
    pub fn create(
        to: &str,
        from: &str,
        exchange_id: &str,
        data: &QuoteData,
        protocol: Option<String>,
        external_id: Option<String>,
    ) -> Result<Self> {
        let metadata = MessageMetadata {
            from: from.to_string(),
            to: to.to_string(),
            kind: MessageKind::Quote,
            id: MessageKind::Quote.typesafe_id()?,
            exchange_id: exchange_id.to_string(),
            external_id,
            protocol: protocol.unwrap_or_else(|| DEFAULT_PROTOCOL_VERSION.to_string()),
            created_at: get_utc_now(),
        };

        let quote = Self {
            metadata: metadata.clone(),
            data: data.clone(),
            signature: String::default(),
        };

        Ok(quote)
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
        crate::json_schemas::validate_from_str(QUOTE_DATA_JSON_SCHEMA, &self.data)?;

        // verify signature
        // TODO
        // crate::signature::verify(
        //     &self.metadata.from,
        //     &serde_json::to_value(self.metadata.clone())?,
        //     &serde_json::to_value(self.data.clone())?,
        //     &self.signature,
        // )?;

        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    pub expires_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub payout_units_per_payin_unit: Option<String>,
    pub payin: QuoteDetails,
    pub payout: QuoteDetails,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_code: String,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub subtotal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]

    pub total: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
}

// TODO: Uncomment when we have parse_quote.json vector updated with no payment instructions
// #[cfg(test)]
// mod tbdex_test_vectors_protocol {
//     use super::*;
//     use std::fs;
//
//     #[derive(Debug, serde::Deserialize)]
//     pub struct TestVector {
//         pub input: String,
//         pub output: Quote,
//     }
//
//
//     #[test]
//     fn parse_quote() {
//         let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-quote.json";
//         let test_vector_json: String = fs::read_to_string(path).unwrap();
//
//         let test_vector: TestVector = serde_json::from_str(&test_vector_json).unwrap();
//         let parsed_quote: Quote = Quote::from_json_string(&test_vector.input).unwrap();
//
//         assert_eq!(test_vector.output, parsed_quote);
//     }
// }
