use super::{generate_access_token, get_service_endpoint, send_request, HttpClientError, Result};
use crate::{http::balances::GetBalancesResponseBody, resources::balance::Balance};
use reqwest::Method;
use web5::dids::bearer_did::BearerDid;

pub fn get_balances(pfi_did_uri: &str, bearer_did: &BearerDid) -> Result<Vec<Balance>> {
    let service_endpoint = get_service_endpoint(pfi_did_uri)?;
    let balances_endpoint = format!("{}/balances", service_endpoint);

    let access_token = generate_access_token(pfi_did_uri, bearer_did)?;

    let balances_response = send_request::<(), GetBalancesResponseBody>(
        &balances_endpoint,
        Method::GET,
        None,
        Some(access_token),
    )?
    .ok_or(HttpClientError::ReqwestError(
        "get balances response returned null".to_string(),
    ))?;

    for balance in &balances_response.data {
        balance.verify()?;
    }

    Ok(balances_response.data)
}
