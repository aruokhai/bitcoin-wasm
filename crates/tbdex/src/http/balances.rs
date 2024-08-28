use crate::{
    json::{FromJson, ToJson},
    resources::balance::Balance,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetBalancesResponseBody {
    pub data: Vec<Balance>,
}
impl FromJson for GetBalancesResponseBody {}
impl ToJson for GetBalancesResponseBody {}
