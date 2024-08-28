use crate::{
    json::{FromJson, ToJson},
    resources::offering::Offering,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetOfferingsResponseBody {
    pub data: Vec<Offering>,
}
impl FromJson for GetOfferingsResponseBody {}
impl ToJson for GetOfferingsResponseBody {}
