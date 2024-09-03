pub mod balances;
pub mod exchanges;
pub mod offerings;

use crate::{
    http::ErrorResponseBody, messages::MessageError, request::{get_scheme, request, SchemeProps}, resources::ResourceError
};
use jwt_compact::{alg::Ed25519, AlgorithmExt, Claims, Header};
// use josekit::{jwt::JwtPayload, JoseError as JosekitError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;
use wasi::{clocks::wall_clock, http::types::Method, random};
use uuid::Uuid;
use crate::web5::dids::{
    bearer_did::{BearerDid, BearerDidError},
    resolution::{
        resolution_metadata::ResolutionMetadataError, resolution_result::ResolutionResult,
    },
};

#[derive(Debug, Serialize, Deserialize)]
struct CustomClaims {
    /// `sub` is a standard claim which denotes claim subject:
    /// https://tools.ietf.org/html/rfc7519#section-4.1.2
    #[serde(rename = "iss")]
    issuer: String,
    #[serde(rename = "jti")]
    jwt_id: String,
    #[serde(rename = "aud")]
    audience: String,


}

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



impl From<SerdeJsonError> for HttpClientError {
    fn from(err: SerdeJsonError) -> Self {
        HttpClientError::SerdeJson(err.to_string())
    }
}



type Result<T> = std::result::Result<T, HttpClientError>;

fn generate_access_token(pfi_did_uri: &str, bearer_did: &BearerDid) -> Result<String> {
    let issud_at_date = wall_clock::now();
    let exp = issud_at_date.seconds + 60;
    let random_number1 = random::random::get_random_u64();
    let random_number2 = random::random::get_random_u64();
    let  mut payload = Claims::new(CustomClaims{
        issuer: bearer_did.did.uri.clone(),
        jwt_id: Uuid::from_u64_pair(random_number1, random_number2).to_string(),
        audience: pfi_did_uri.to_string()
        
    });
    let key_id = bearer_did.document.verification_method[0].id.clone();
    payload.expiration = chrono::DateTime::from_timestamp(exp as i64, 0); 
    payload.issued_at = chrono::DateTime::from_timestamp(issud_at_date.seconds as i64, issud_at_date.nanoseconds);
    let web5_signer = bearer_did.get_signer(bearer_did.document.verification_method[0].id.clone())?;
    let  header = Header::empty()
    .with_token_type("JWT")
    .with_key_id(key_id);
    let access_token = Ed25519.token(&header, &payload, &web5_signer.get_signing_key().unwrap()).unwrap();


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
    let SchemeProps { url_scheme, url, url_path } = get_scheme(url);
    let mut additional_headers = None;
    if let Some(token) = &access_token {
        let token = format!("Bearer {token}");
        additional_headers = Some(vec![("authorization".to_string(),token.as_bytes().to_vec() ),
           ("content-type".to_string(),b"application/json".to_vec()) ]);
    }

    let request_body = if let Some(body) = &body {
        let parsed_body = serde_json::to_vec(body).unwrap();
        println!("this is body {:?}", String::from_utf8(parsed_body.clone()).unwrap());

        Some(parsed_body)
    } else  {
        None
    };
    let response = request(method, url_scheme, &url, &url_path, request_body.as_deref(), additional_headers.as_deref(), None, None, None).unwrap();
    let response_status = response.status;
    let response_text = response.body;
   // println!("response status {:?}", response_text);

    // crate::log_dbg!(|| {
    //     format!(
    //         "httpclient sent request {} {}, has access token {}, with body {}, \
    //         response status {}, response text {}",
    //         method,
    //         url,
    //         access_token.is_some(),
    //         match &body {
    //             Some(b) => serde_json::to_string_pretty(b)
    //                 .unwrap_or_else(|_| String::from("error serializing the body")),
    //             None => String::default(),
    //         },
    //         response_status,
    //         match serde_json::from_str::<serde_json::Value>(&response_text) {
    //             Ok(json) =>
    //                 serde_json::to_string_pretty(&json).unwrap_or_else(|_| response_text.clone()),
    //             Err(_) => response_text.clone(),
    //         }
    //     )
    // });

    if  !(300 > response_status && response_status >=  200) {
        if response_status >= 400 {
            let error_string = String::from_utf8(response_text.clone()).unwrap();
            println!("error body {:?}", error_string);
            let error_response_body = serde_json::from_slice::<ErrorResponseBody>(&response_text)?;
            
            return Err(HttpClientError::ErrorResponseBody(error_response_body));
        }

        return Err(HttpClientError::UnsuccessfulResponse(format!(
            "{} {}",
            response_status, "unsuccessful".to_string()        )));
    }

    if response_status == 202 {
        return Ok(None);
    }
    let error_string = String::from_utf8(response_text.clone()).unwrap();
    println!("result body {:?}", error_string);
    let response_body = serde_json::from_slice(&response_text)?;
    Ok(Some(response_body))
}
