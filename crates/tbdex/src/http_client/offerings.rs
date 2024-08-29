use wasi::http::types::Method;

use super::{get_service_endpoint, send_request, HttpClientError, Result};
use crate::{http::offerings::GetOfferingsResponseBody, resources::offering::Offering};

pub fn get_offerings(pfi_did_uri: &str) -> Result<Vec<Offering>> {
    let service_endpoint = get_service_endpoint(pfi_did_uri).unwrap();
    let offerings_endpoint = format!("{}/offerings", service_endpoint);
    let offerings_response =
        send_request::<(), GetOfferingsResponseBody>(&offerings_endpoint, Method::Get, None, None)?
            .ok_or(HttpClientError::ReqwestError(
                "get offerings response returned null".to_string(),
            )).unwrap();

    // TODO Cancellation details missing
    // for offering in &offerings_response.data {
    //     offering.verify()?;
    // }

    Ok(offerings_response.data)
}
