pub mod balance;
pub mod offering;

use crate::{json_schemas::JsonSchemaError, signature::SignatureError};
use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use type_safe_id::{DynamicType, Error as TypeIdError, TypeSafeId};
use web5::dids::bearer_did::BearerDidError;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ResourceError {
    #[error("serde json error {0}")]
    SerdeJson(String),
    #[error("typeid error {0}")]
    TypeId(String),
    #[error(transparent)]
    BearerDid(#[from] BearerDidError),
    #[error(transparent)]
    Signature(#[from] SignatureError),
    #[error(transparent)]
    JsonSchema(#[from] JsonSchemaError),
}

impl From<SerdeJsonError> for ResourceError {
    fn from(err: SerdeJsonError) -> Self {
        ResourceError::SerdeJson(err.to_string())
    }
}

impl From<TypeIdError> for ResourceError {
    fn from(err: TypeIdError) -> Self {
        ResourceError::TypeId(err.to_string())
    }
}

type Result<T> = std::result::Result<T, ResourceError>;

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    Offering,
    Balance,
}

impl ResourceKind {
    pub fn typesafe_id(&self) -> Result<String> {
        let serialized_kind = serde_json::to_string(&self)?;
        let dynamic_type = DynamicType::new(serialized_kind.trim_matches('"'))?;
        Ok(TypeSafeId::new_with_type(dynamic_type).to_string())
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub kind: ResourceKind,
    pub from: String,
    pub id: String,
    pub protocol: String,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}
