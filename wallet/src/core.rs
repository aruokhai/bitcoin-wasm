use std::collections::BTreeSet;

use bdk::{wallet::AddressIndex, Wallet};
use bdk_file_store::Store;

pub struct NodeWallet  {
    pub inner: Wallet<Store<BTreeSet<String>>>,
}

impl NodeWallet {

    pub fn get_newaddress(&self) -> String {
        let newaddress = self.inner.get_address(AddressIndex::New).
    }
    
}