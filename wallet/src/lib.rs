#[allow(warnings)]
mod bindings;

use core::NodeWallet;
use std::collections::BTreeSet;

use bdk_file_store::Store;
use bindings::Guest;
use bdk::{bitcoin::{bip32::DerivationPath, Network}, descriptor, keys::{bip39::{Language, Mnemonic}, DerivableKey}, wallet, Wallet};
use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::descriptor::error::Error;
struct Component;
enum NetworkType {
    testnet,
    mainnet,
}

mod core;
mod store;
mod database;

type TestChangeSet = BTreeSet<String>;

impl Guest for Component {
   
    fn create_wallet(mnemomic_str: &str,filestore_path: &str, network:  NetworkType) -> NodeWallet {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemomic_str).unwrap();
        let extended_key = mnemonic.into_extended_key().unwrap();
        let network = match network {
            NetworkType::mainnet => Network::Bitcoin,
            NetworkType::testnet => Network::Testnet,
        };
        let xprv = extended_key.into_xprv(network).unwrap().to_string();
        let receive_descriptor = format!("wpkh({}/84'/1'/0'/0/*)",xprv);
        let change_descriptor = format!("wpkh({}/84'/1'/0'/1/*)",xprv);
        let store = Store::<TestChangeSet>::create_new(b"wasm_wallet", filestore_path).unwrap();
        let wallet = Wallet::new(receive_descriptor, Some(change_descriptor), network, store).unwrap();
        return NodeWallet { inner: wallet }

    }
}

bindings::export!(Component with_types_in bindings);
