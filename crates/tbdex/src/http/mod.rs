pub mod balances;
pub mod exchanges;
pub mod offerings;

use crate::json::{FromJson, ToJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponseBody {
    pub message: String,
    pub details: Option<Vec<ErrorDetail>>,
}
impl FromJson for ErrorResponseBody {}
impl ToJson for ErrorResponseBody {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub id: Option<String>,
    pub message: Option<String>,
    pub path: Option<String>,
}

impl std::fmt::Display for ErrorResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = self.message.clone();

        if let Some(details) = &self.details {
            output.push_str(" [");
            for (i, detail) in details.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str("detail: ");
                if let Some(id) = &detail.id {
                    output.push_str(&format!("id: {}, ", id));
                }
                if let Some(message) = &detail.message {
                    output.push_str(&format!("message: {}, ", message));
                }
                if let Some(path) = &detail.path {
                    output.push_str(&format!("path: {}", path));
                }
            }
            output.push(']');
        }

        write!(f, "{}", output)
    }
}

impl std::error::Error for ErrorResponseBody {}
