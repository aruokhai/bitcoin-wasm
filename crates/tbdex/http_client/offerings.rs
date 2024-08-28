use super::{get_service_endpoint, send_request, HttpClientError, Result};
use crate::{http::offerings::GetOfferingsResponseBody, resources::offering::Offering};
use reqwest::Method;

pub fn get_offerings(pfi_did_uri: &str) -> Result<Vec<Offering>> {
    let service_endpoint = get_service_endpoint(pfi_did_uri)?;
    let offerings_endpoint = format!("{}/offerings", service_endpoint);

    let offerings_response =
        send_request::<(), GetOfferingsResponseBody>(&offerings_endpoint, Method::GET, None, None)?
            .ok_or(HttpClientError::ReqwestError(
                "get offerings response returned null".to_string(),
            ))?;

    for offering in &offerings_response.data {
        offering.verify()?;
    }

    Ok(offerings_response.data)
}
