use crate::{
    json::{FromJson, ToJson},
    messages::{
        cancel::Cancel, close::Close, order::Order, order_instructions::OrderInstructions,
        order_status::OrderStatus, quote::Quote, rfq::Rfq, Message, MessageKind,
    },
};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{str::FromStr, sync::Arc};

#[derive(Serialize, Deserialize)]
pub struct GetExchangeResponseBody {
    pub data: Vec<Message>,
}
impl FromJson for GetExchangeResponseBody {}
impl ToJson for GetExchangeResponseBody {}

#[derive(Serialize, Deserialize)]
pub struct GetExchangesResponseBody {
    pub data: Vec<String>,
}
impl FromJson for GetExchangesResponseBody {}
impl ToJson for GetExchangesResponseBody {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExchangeRequestBody {
    pub message: Rfq,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}
impl FromJson for CreateExchangeRequestBody {}
impl ToJson for CreateExchangeRequestBody {}

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum WalletUpdateMessage {
    Order(Arc<Order>),
    Cancel(Arc<Cancel>),
}
impl FromJson for WalletUpdateMessage {}
impl ToJson for WalletUpdateMessage {}

impl<'de> Deserialize<'de> for WalletUpdateMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageVisitor;

        impl<'de> Visitor<'de> for MessageVisitor {
            type Value = WalletUpdateMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an Order or Cancel")
            }

            fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

                let kind_str = value
                    .get("metadata")
                    .and_then(|m| m.get("kind"))
                    .and_then(|k| k.as_str());

                match kind_str {
                    Some(k) => match MessageKind::from_str(k) {
                        Ok(kind) => match kind {
                            MessageKind::Order => {
                                if let Ok(order) = serde_json::from_value::<Order>(value.clone()) {
                                    Ok(WalletUpdateMessage::Order(Arc::new(order)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize order"))
                                }
                            }
                            MessageKind::Cancel => {
                                if let Ok(cancel) = serde_json::from_value::<Cancel>(value.clone())
                                {
                                    Ok(WalletUpdateMessage::Cancel(Arc::new(cancel)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize cancel"))
                                }
                            }
                            _ => Err(serde::de::Error::custom(format!(
                                "unexpected message kind {:?}",
                                kind
                            ))),
                        },
                        Err(_) => Err(serde::de::Error::custom(format!(
                            "unexpected message kind {}",
                            k
                        ))),
                    },
                    None => Err(serde::de::Error::custom(format!(
                        "unexpected message kind {:?}",
                        kind_str
                    ))),
                }
            }

            fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Err(serde::de::Error::custom("message is missing"))
            }
        }

        deserializer.deserialize_option(MessageVisitor)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UpdateExchangeRequestBody {
    pub message: WalletUpdateMessage,
}
impl FromJson for UpdateExchangeRequestBody {}
impl ToJson for UpdateExchangeRequestBody {}

#[derive(Serialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum ReplyToMessage {
    Quote(Arc<Quote>),
    OrderStatus(Arc<OrderStatus>),
    OrderInstructions(Arc<OrderInstructions>),
    Close(Arc<Close>),
}
impl FromJson for ReplyToMessage {}
impl ToJson for ReplyToMessage {}

impl<'de> Deserialize<'de> for ReplyToMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageVisitor;

        impl<'de> Visitor<'de> for MessageVisitor {
            type Value = ReplyToMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an Order or Cancel")
            }

            fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

                let kind_str = value
                    .get("metadata")
                    .and_then(|m| m.get("kind"))
                    .and_then(|k| k.as_str());

                match kind_str {
                    Some(k) => match MessageKind::from_str(k) {
                        Ok(kind) => match kind {
                            MessageKind::Quote => {
                                if let Ok(quote) = serde_json::from_value::<Quote>(value.clone()) {
                                    Ok(ReplyToMessage::Quote(Arc::new(quote)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize quote"))
                                }
                            }
                            MessageKind::OrderStatus => {
                                if let Ok(order_status) =
                                    serde_json::from_value::<OrderStatus>(value.clone())
                                {
                                    Ok(ReplyToMessage::OrderStatus(Arc::new(order_status)))
                                } else {
                                    Err(serde::de::Error::custom(
                                        "failed to deserialize order_status",
                                    ))
                                }
                            }
                            MessageKind::OrderInstructions => {
                                if let Ok(order_instructions) =
                                    serde_json::from_value::<OrderInstructions>(value.clone())
                                {
                                    Ok(ReplyToMessage::OrderInstructions(Arc::new(
                                        order_instructions,
                                    )))
                                } else {
                                    Err(serde::de::Error::custom(
                                        "failed to deserialize order_instructions",
                                    ))
                                }
                            }
                            MessageKind::Close => {
                                if let Ok(close) = serde_json::from_value::<Close>(value.clone()) {
                                    Ok(ReplyToMessage::Close(Arc::new(close)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize close"))
                                }
                            }
                            _ => Err(serde::de::Error::custom(format!(
                                "unexpected message kind {:?}",
                                kind
                            ))),
                        },
                        Err(_) => Err(serde::de::Error::custom(format!(
                            "unexpected message kind {}",
                            k
                        ))),
                    },
                    None => Err(serde::de::Error::custom(format!(
                        "unexpected message kind {:?}",
                        kind_str
                    ))),
                }
            }

            fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Err(serde::de::Error::custom("message is missing"))
            }
        }

        deserializer.deserialize_option(MessageVisitor)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ReplyToRequestBody {
    pub message: ReplyToMessage,
}
impl FromJson for ReplyToRequestBody {}
impl ToJson for ReplyToRequestBody {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[derive(Debug, serde::Deserialize)]
    pub struct TestVector<T> {
        pub input: String,
        pub output: T,
    }

    #[test]
    fn order_update_exchange_request_body() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-order.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector = serde_json::from_str::<TestVector<Order>>(&test_vector_json).unwrap();
        let parsed_order = Order::from_json_string(&test_vector.input).unwrap();

        let update_exchange_request_body = UpdateExchangeRequestBody {
            message: WalletUpdateMessage::Order(Arc::new(parsed_order)),
        };

        let serialized = update_exchange_request_body.to_json_string().unwrap();
        let deserialized = UpdateExchangeRequestBody::from_json_string(&serialized).unwrap();

        assert_eq!(update_exchange_request_body, deserialized);
    }

    #[test]
    fn cancel_update_exchange_request_body() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-cancel.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector = serde_json::from_str::<TestVector<Cancel>>(&test_vector_json).unwrap();
        let parsed_cancel = Cancel::from_json_string(&test_vector.input).unwrap();

        let update_exchange_request_body = UpdateExchangeRequestBody {
            message: WalletUpdateMessage::Cancel(Arc::new(parsed_cancel)),
        };

        let serialized = update_exchange_request_body.to_json_string().unwrap();
        let deserialized = UpdateExchangeRequestBody::from_json_string(&serialized).unwrap();

        assert_eq!(update_exchange_request_body, deserialized);
    }

    #[test]
    fn quote_reply_to_request_body() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-quote.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector = serde_json::from_str::<TestVector<Quote>>(&test_vector_json).unwrap();
        let parsed_quote = Quote::from_json_string(&test_vector.input).unwrap();

        let reply_to_request_body = ReplyToRequestBody {
            message: ReplyToMessage::Quote(Arc::new(parsed_quote)),
        };

        let serialized = reply_to_request_body.to_json_string().unwrap();
        let deserialized = ReplyToRequestBody::from_json_string(&serialized).unwrap();

        assert_eq!(reply_to_request_body, deserialized);
    }

    #[test]
    fn order_status_reply_to_request_body() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-orderstatus.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector =
            serde_json::from_str::<TestVector<OrderStatus>>(&test_vector_json).unwrap();
        let parsed_order_status = OrderStatus::from_json_string(&test_vector.input).unwrap();

        let reply_to_request_body = ReplyToRequestBody {
            message: ReplyToMessage::OrderStatus(Arc::new(parsed_order_status)),
        };

        let serialized = reply_to_request_body.to_json_string().unwrap();
        let deserialized = ReplyToRequestBody::from_json_string(&serialized).unwrap();

        assert_eq!(reply_to_request_body, deserialized);
    }

    #[test]
    fn close_reply_to_request_body() {
        let path = "../../tbdex/hosted/test-vectors/protocol/vectors/parse-close.json";
        let test_vector_json: String = fs::read_to_string(path).unwrap();

        let test_vector = serde_json::from_str::<TestVector<Close>>(&test_vector_json).unwrap();
        let parsed_close = Close::from_json_string(&test_vector.input).unwrap();

        let reply_to_request_body = ReplyToRequestBody {
            message: ReplyToMessage::Close(Arc::new(parsed_close)),
        };

        let serialized = reply_to_request_body.to_json_string().unwrap();
        let deserialized = ReplyToRequestBody::from_json_string(&serialized).unwrap();

        assert_eq!(reply_to_request_body, deserialized);
    }
}
