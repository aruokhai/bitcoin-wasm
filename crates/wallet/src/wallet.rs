use bitcoin::{bip32::Xpub, hashes::Hash, key::Secp256k1, Address, Amount, FeeRate, Network, Script};

use crate::{coin_selection::{CoinSelectionAlgorithm, DefaultCoinSelectionAlgorithm}, errors, types};
use wasi::random::{self, random::{get_random_u64, get_random_bytes}};

use rand_core::RngCore;



#[derive(Copy, Clone)]
pub enum WalletType {
    P2WPKH,
}



struct WasiRandom;

impl RngCore for WasiRandom {
    fn next_u32(&mut self) -> u32 {
        get_random_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        get_random_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let source = get_random_bytes(dest.len() as u64);
        dest[..source.len()].copy_from_slice(&source);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }
}

pub struct  Wallet {
    master_public: Xpub,
    network: Network,
    utxos: Vec<types::WeightedUtxo>,
    wallet_type: WalletType,
    receive_depth: u32,
    change_depth: u32,

}

pub struct AddressDetails {
    hash: Vec<u8>,
    human: String,
}

impl Wallet {

    pub fn new(master_public: Xpub, network: Network) -> Self {
        Wallet {
            master_public,
            network,
            utxos: Vec::new(),
            wallet_type: WalletType::P2WPKH,
            receive_depth: 0,
            change_depth: 0
            
        }
    }

    pub fn add_utxo(&mut self, utxo: types::WeightedUtxo) {
        self.utxos.push(utxo);
    }

    pub fn derive_p2wpkh_receive_address(& mut self) -> Result< AddressDetails ,errors::CreateTxError>{
        let secp = Secp256k1::new();
        let child_pub = self.master_public
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: 0 })
            .map_err(|err| errors::CreateTxError::PubKeyError(err) )?
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: self.receive_depth })
            .map_err(|err| errors::CreateTxError::PubKeyError(err) )?.to_pub();

        self.receive_depth +=1;    
        let pub_key = Address::p2wpkh(&child_pub, self.network)
            .pubkey_hash()
            .ok_or(errors::CreateTxError::NoPubKey)?;

        let hash =  pub_key.to_raw_hash().as_byte_array().to_vec();

        return  Ok(AddressDetails { hash, human: pub_key.to_string() })
        
    }

    fn derive_p2wpkh_change_script(& mut self) -> Result< Vec<u8> ,errors::CreateTxError>{
        let secp = Secp256k1::new();
        let child_pub = self.master_public
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: 1 })
            .map_err(|err| errors::CreateTxError::PubKeyError(err) )?
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: self.change_depth })
            .map_err(|err| errors::CreateTxError::PubKeyError(err) )?.to_pub();

        self.change_depth +=1;    
        let pub_key = Address::p2wpkh(&child_pub, self.network)
            .script_pubkey();
            
        return  Ok(pub_key.to_bytes().to_vec())
        
    }

    pub fn send_tx(&self, recipients: Vec<u8>, fee_rate: FeeRate, amount: Amount) -> Result<bitcoin::Transaction, errors::CreateTxError> {
        let change_script = self.derive_p2wpkh_change_script()?;
        let coinselection = DefaultCoinSelectionAlgorithm::default().coin_select(vec![], self.utxos, fee_rate, amount, Script::from_bytes(&change_script), &mut WasiRandom);
    }
}



