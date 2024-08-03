use std::sync::Arc;

use super::{get_service_endpoint, send_request, Result};
use crate::http::exchanges::GetExchangesResponseBody;
use crate::{
    http::exchanges::{
        CreateExchangeRequestBody, GetExchangeResponseBody, UpdateExchangeRequestBody,
        WalletUpdateMessage,
    },
    http_client::{generate_access_token, HttpClientError},
    messages::{
        cancel::Cancel, close::Close, order::Order, order_instructions::OrderInstructions,
        order_status::OrderStatus, quote::Quote, rfq::Rfq, Message,
    },
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use web5::dids::bearer_did::BearerDid;

#[derive(Clone, Default, Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Exchange {
    pub rfq: Arc<Rfq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<Arc<Quote>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Arc<Order>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_instructions: Option<Arc<OrderInstructions>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel: Option<Arc<Cancel>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_statuses: Option<Vec<Arc<OrderStatus>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<Arc<Close>>,
}

pub fn create_exchange(rfq: &Rfq, reply_to: Option<String>) -> Result<()> {
    let service_endpoint = get_service_endpoint(&rfq.metadata.to)?;
    let create_exchange_endpoint = format!("{}/exchanges", service_endpoint);

    rfq.verify()?;

    send_request::<CreateExchangeRequestBody, ()>(
        &create_exchange_endpoint,
        Method::POST,
        Some(&CreateExchangeRequestBody {
            message: rfq.clone(),
            reply_to,
        }),
        None,
    )?;

    Ok(())
}

pub fn submit_order(order: &Order) -> Result<()> {
    let service_endpoint = get_service_endpoint(&order.metadata.to)?;
    let submit_order_endpoint = format!(
        "{}/exchanges/{}",
        service_endpoint, order.metadata.exchange_id
    );

    order.verify()?;

    send_request::<UpdateExchangeRequestBody, ()>(
        &submit_order_endpoint,
        Method::PUT,
        Some(&UpdateExchangeRequestBody {
            message: WalletUpdateMessage::Order(Arc::new(order.clone())),
        }),
        None,
    )?;

    Ok(())
}

pub fn submit_cancel(cancel: &Cancel) -> Result<()> {
    let service_endpoint = get_service_endpoint(&cancel.metadata.to)?;
    let submit_cancel_endpoint = format!(
        "{}/exchanges/{}",
        service_endpoint, cancel.metadata.exchange_id
    );

    cancel.verify()?;

    send_request::<UpdateExchangeRequestBody, ()>(
        &submit_cancel_endpoint,
        Method::PUT,
        Some(&UpdateExchangeRequestBody {
            message: WalletUpdateMessage::Cancel(Arc::new(cancel.clone())),
        }),
        None,
    )?;

    Ok(())
}

pub fn get_exchange(
    pfi_did_uri: &str,
    bearer_did: &BearerDid,
    exchange_id: &str,
) -> Result<Exchange> {
    let service_endpoint = get_service_endpoint(pfi_did_uri)?;
    let get_exchange_endpoint = format!("{}/exchanges/{}", service_endpoint, exchange_id);

    let access_token = generate_access_token(pfi_did_uri, bearer_did)?;

    let response = send_request::<(), GetExchangeResponseBody>(
        &get_exchange_endpoint,
        Method::GET,
        None,
        Some(access_token),
    )?
    .ok_or(HttpClientError::ReqwestError(
        "get exchange response cannot be null".to_string(),
    ))?;

    let mut exchange = Exchange::default();

    for message in response.data {
        match message {
            Message::Rfq(rfq) => {
                exchange.rfq = rfq;
            }
            Message::Quote(quote) => {
                exchange.quote = Some(quote);
            }
            Message::Order(order) => {
                exchange.order = Some(order);
            }
            Message::OrderInstructions(order_instructions) => {
                exchange.order_instructions = Some(order_instructions);
            }
            Message::Cancel(cancel) => {
                exchange.cancel = Some(cancel);
            }
            Message::OrderStatus(order_status) => {
                if let Some(order_statuses) = &mut exchange.order_statuses {
                    order_statuses.push(order_status);
                } else {
                    exchange.order_statuses = Some(vec![order_status]);
                }
            }
            Message::Close(close) => {
                exchange.close = Some(close);
            }
        }
    }

    Ok(exchange)
}

pub fn get_exchange_ids(pfi_did: &str, requestor_did: &BearerDid) -> Result<Vec<String>> {
    let service_endpoint = get_service_endpoint(pfi_did)?;
    let get_exchanges_endpoint = format!("{}/exchanges", service_endpoint);

    let access_token = generate_access_token(pfi_did, requestor_did)?;

    let response = send_request::<(), GetExchangesResponseBody>(
        &get_exchanges_endpoint,
        Method::GET,
        None,
        Some(access_token),
    )?
    .ok_or(HttpClientError::ReqwestError(
        "get exchanges response cannot be null".to_string(),
    ))?;

    Ok(response.data)
}
