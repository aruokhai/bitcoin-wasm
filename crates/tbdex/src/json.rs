use serde::{de::DeserializeOwned, Serialize};
use serde_json::Error as SerdeJsonError;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum JsonError {
    #[error("serde json error {0}")]
    SerdeJson(String),
}

impl From<SerdeJsonError> for JsonError {
    fn from(err: SerdeJsonError) -> Self {
        JsonError::SerdeJson(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, JsonError>;

pub trait FromJson: Sized + DeserializeOwned {
    fn from_json_string(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(JsonError::from)
    }
}

pub trait ToJson: Serialize {
    fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(JsonError::from)
    }
}
