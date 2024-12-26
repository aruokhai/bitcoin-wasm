#[allow(warnings)]
mod bindings;

use bindings::Guest;
use bitcoin::{bip32::Xpub, Network};


mod coin_selection;
mod utils;
mod types;
mod errors;

mod tx_builder;

#[derive(Copy, Clone)]
pub enum WalletType {
    P2WPKH,
}

pub struct  Wallet {
    master_public: Xpub,
    network: Network,
    utxos: Vec<types::WeightedUtxo>,
    wallet_type: WalletType,
}

impl Wallet {

    pub fn new(master_public: Xpub, network: Network) -> Self {
        Wallet {
            master_public,
            network,
            utxos: Vec::new(),
            wallet_type: WalletType::P2WPKH
            
        }
    }

    pub fn add_utxo(&mut self, utxo: types::WeightedUtxo) {
        self.utxos.push(utxo);
    }

    pub fn build_tx(&self, recipients: Vec<(String, u64)>) -> Result<bitcoin::Transaction, errors::CreateTxError> {
        tx_builder::build_tx(&self.master_public, &self.utxos, &recipients, self.network)
    }
}




struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
