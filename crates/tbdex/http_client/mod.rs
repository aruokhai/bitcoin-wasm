pub mod balances;
pub mod exchanges;
pub mod offerings;

use crate::{
    http::ErrorResponseBody, jose::Signer, messages::MessageError, resources::ResourceError,
};
use josekit::{jwt::JwtPayload, JoseError as JosekitError};
use reqwest::{blocking::Client, Error as ReqwestError, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Error as SerdeJsonError;
use std::time::{Duration, SystemTime};
use uuid::Uuid;
use web5::dids::{
    bearer_did::{BearerDid, BearerDidError},
    resolution::{
        resolution_metadata::ResolutionMetadataError, resolution_result::ResolutionResult,
    },
};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum HttpClientError {
    #[error("reqwest error {0}")]
    ReqwestError(String),
    #[error("serde json error {0}")]
    SerdeJson(String),
    #[error(transparent)]
    BearerDid(#[from] BearerDidError),
    #[error("jose error {0}")]
    Jose(String),
    #[error(transparent)]
    Resource(#[from] ResourceError),
    #[error(transparent)]
    Message(#[from] MessageError),
    #[error("unable to map response to exchange")]
    ExchangeMapping,
    #[error(transparent)]
    Resolution(#[from] ResolutionMetadataError),
    #[error("missing service endpoint for {0}")]
    MissingServiceEndpoint(String),
    #[error("unsuccessfuly response {0}")]
    UnsuccessfulResponse(String),
    #[error(transparent)]
    ErrorResponseBody(#[from] ErrorResponseBody),
}

impl From<ReqwestError> for HttpClientError {
    fn from(err: ReqwestError) -> Self {
        HttpClientError::ReqwestError(err.to_string())
    }
}

impl From<SerdeJsonError> for HttpClientError {
    fn from(err: SerdeJsonError) -> Self {
        HttpClientError::SerdeJson(err.to_string())
    }
}

impl From<JosekitError> for HttpClientError {
    fn from(err: JosekitError) -> Self {
        HttpClientError::Jose(err.to_string())
    }
}

type Result<T> = std::result::Result<T, HttpClientError>;

fn generate_access_token(pfi_did_uri: &str, bearer_did: &BearerDid) -> Result<String> {
    let now = SystemTime::now();
    let exp = now + Duration::from_secs(60);

    let mut payload = JwtPayload::new();
    payload.set_audience(vec![pfi_did_uri]);
    payload.set_issuer(&bearer_did.did.uri);
    payload.set_issued_at(&now);
    payload.set_expires_at(&exp);
    payload.set_jwt_id(Uuid::new_v4().to_string());

    // default to first VM
    let key_id = bearer_did.document.verification_method[0].id.clone();
    let web5_signer = bearer_did.get_signer(key_id.clone())?;
    let jose_signer = Signer {
        kid: key_id,
        web5_signer,
    };

    let access_token = jose_signer.sign_jwt(&payload)?;

    Ok(access_token)
}

pub(crate) fn get_service_endpoint(pfi_did_uri: &str) -> Result<String> {
    let resolution_result = ResolutionResult::new(pfi_did_uri);

    let endpoint = match &resolution_result.document {
        None => {
            return Err(match resolution_result.resolution_metadata.error {
                Some(e) => HttpClientError::Resolution(e),
                None => HttpClientError::Resolution(ResolutionMetadataError::InternalError),
            })
        }
        Some(d) => match &d.service {
            None => {
                return Err(HttpClientError::MissingServiceEndpoint(
                    pfi_did_uri.to_string(),
                ))
            }
            Some(s) => s
                .iter()
                .find(|s| s.r#type == *"PFI")
                .ok_or(HttpClientError::MissingServiceEndpoint(
                    pfi_did_uri.to_string(),
                ))?
                .service_endpoint
                .first()
                .ok_or(HttpClientError::MissingServiceEndpoint(
                    pfi_did_uri.to_string(),
                ))?
                .clone(),
        },
    };

    Ok(endpoint)
}

fn send_request<T: Serialize, U: DeserializeOwned>(
    url: &str,
    method: Method,
    body: Option<&T>,
    access_token: Option<String>,
) -> Result<Option<U>> {
    let client = Client::new();
    let mut request = client.request(method.clone(), url);

    if let Some(token) = &access_token {
        request = request.bearer_auth(token);
    }

    if let Some(body) = &body {
        request = request.json(body);
    }

    let response = request.send()?;

    let response_status = response.status();
    let response_text = response.text()?;

    crate::log_dbg!(|| {
        format!(
            "httpclient sent request {} {}, has access token {}, with body {}, \
            response status {}, response text {}",
            method,
            url,
            access_token.is_some(),
            match &body {
                Some(b) => serde_json::to_string_pretty(b)
                    .unwrap_or_else(|_| String::from("error serializing the body")),
                None => String::default(),
            },
            response_status,
            match serde_json::from_str::<serde_json::Value>(&response_text) {
                Ok(json) =>
                    serde_json::to_string_pretty(&json).unwrap_or_else(|_| response_text.clone()),
                Err(_) => response_text.clone(),
            }
        )
    });

    if !response_status.is_success() {
        if response_status.as_u16() >= 400 {
            let error_response_body = serde_json::from_str::<ErrorResponseBody>(&response_text)?;
            return Err(HttpClientError::ErrorResponseBody(error_response_body));
        }

        return Err(HttpClientError::UnsuccessfulResponse(format!(
            "{} {}",
            response_status, response_text
        )));
    }

    if response_status == StatusCode::ACCEPTED {
        return Ok(None);
    }

    let response_body = serde_json::from_str::<U>(&response_text)?;
    Ok(Some(response_body))
}
