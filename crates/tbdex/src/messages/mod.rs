pub mod cancel;
pub mod close;
pub mod order;
pub mod order_instructions;
pub mod order_status;
pub mod quote;
pub mod rfq;

use crate::{
    json::{FromJson, ToJson},
    json_schemas::JsonSchemaError,
    signature::SignatureError,
};
use cancel::Cancel;
use close::Close;
use order::Order;
use order_instructions::OrderInstructions;
use order_status::OrderStatus;
use quote::Quote;
use rfq::Rfq;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use serde_json::Error as SerdeJsonError;
use std::{str::FromStr, sync::Arc};
use type_safe_id::{DynamicType, Error as TypeIdError, TypeSafeId};
use web5::dids::bearer_did::BearerDidError;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum MessageError {
    #[error("serde json error {0}")]
    SerdeJson(String),
    #[error("typeid error {0}")]
    TypeId(String),
    #[error(transparent)]
    BearerDid(#[from] BearerDidError),
    #[error(transparent)]
    Signature(#[from] SignatureError),
    #[error("unknown kind {0}")]
    UnknownKind(String),
    #[error("offering verification failure {0}")]
    OfferingVerification(String),
    #[error(transparent)]
    JsonSchema(#[from] JsonSchemaError),
    #[error("private data verification failure {0}")]
    PrivateDataVerification(String),
}

impl From<SerdeJsonError> for MessageError {
    fn from(err: SerdeJsonError) -> Self {
        MessageError::SerdeJson(err.to_string())
    }
}

impl From<TypeIdError> for MessageError {
    fn from(err: TypeIdError) -> Self {
        MessageError::TypeId(err.to_string())
    }
}

type Result<T> = std::result::Result<T, MessageError>;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MessageKind {
    #[default]
    Rfq,
    Quote,
    Order,
    OrderInstructions,
    Cancel,
    OrderStatus,
    Close,
}

impl MessageKind {
    pub fn typesafe_id(&self) -> Result<String> {
        let serialized_kind = serde_json::to_string(&self)?;
        let dynamic_type = DynamicType::new(serialized_kind.trim_matches('"'))?;
        Ok(TypeSafeId::new_with_type(dynamic_type).to_string())
    }
}

impl FromStr for MessageKind {
    type Err = MessageError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "rfq" => Ok(MessageKind::Rfq),
            "quote" => Ok(MessageKind::Quote),
            "order" => Ok(MessageKind::Order),
            "orderinstructions" => Ok(MessageKind::OrderInstructions),
            "cancel" => Ok(MessageKind::Cancel),
            "orderstatus" => Ok(MessageKind::OrderStatus),
            "close" => Ok(MessageKind::Close),
            _ => Err(MessageError::UnknownKind(s.to_string())),
        }
    }
}

#[derive(Debug, Deserialize, Default, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageMetadata {
    pub from: String,
    pub to: String,
    pub kind: MessageKind,
    pub id: String,
    pub exchange_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub protocol: String,
    pub created_at: String,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Message {
    Rfq(Arc<Rfq>),
    Quote(Arc<Quote>),
    Order(Arc<Order>),
    OrderInstructions(Arc<OrderInstructions>),
    Cancel(Arc<Cancel>),
    OrderStatus(Arc<OrderStatus>),
    Close(Arc<Close>),
}

impl ToJson for Message {}
impl FromJson for Message {}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageVisitor;

        impl<'de> Visitor<'de> for MessageVisitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    "an Rfq, Order, OrderInstructions, OrderStatus, Close, Cancel, or String",
                )
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
                            MessageKind::Rfq => {
                                if let Ok(rfq) = serde_json::from_value::<Rfq>(value.clone()) {
                                    Ok(Message::Rfq(Arc::new(rfq)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize rfq"))
                                }
                            }
                            MessageKind::Quote => {
                                if let Ok(quote) = serde_json::from_value::<Quote>(value.clone()) {
                                    Ok(Message::Quote(Arc::new(quote)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize quote"))
                                }
                            }
                            MessageKind::Order => {
                                if let Ok(order) = serde_json::from_value::<Order>(value.clone()) {
                                    Ok(Message::Order(Arc::new(order)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize order"))
                                }
                            }
                            MessageKind::OrderInstructions => {
                                if let Ok(order_instructions) =
                                    serde_json::from_value::<OrderInstructions>(value.clone())
                                {
                                    Ok(Message::OrderInstructions(Arc::new(order_instructions)))
                                } else {
                                    Err(serde::de::Error::custom(
                                        "failed to deserialize order_instructions",
                                    ))
                                }
                            }
                            MessageKind::Cancel => {
                                if let Ok(cancel) = serde_json::from_value::<Cancel>(value.clone())
                                {
                                    Ok(Message::Cancel(Arc::new(cancel)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize cancel"))
                                }
                            }
                            MessageKind::OrderStatus => {
                                if let Ok(order_status) =
                                    serde_json::from_value::<OrderStatus>(value.clone())
                                {
                                    Ok(Message::OrderStatus(Arc::new(order_status)))
                                } else {
                                    Err(serde::de::Error::custom(
                                        "failed to deserialize order_status",
                                    ))
                                }
                            }
                            MessageKind::Close => {
                                if let Ok(close) = serde_json::from_value::<Close>(value.clone()) {
                                    Ok(Message::Close(Arc::new(close)))
                                } else {
                                    Err(serde::de::Error::custom("failed to deserialize close"))
                                }
                            }
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
