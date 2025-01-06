use std::{collections::BTreeMap, vec};

use bitcoin::{absolute::LockTime, bip32::{ChildNumber, DerivationPath, Xpub}, key::Secp256k1, psbt::{self, Input, PsbtSighashType}, transaction::Version, Address, Amount, CompressedPublicKey, EcdsaSighashType, FeeRate, Network, Psbt, Script, ScriptBuf, Transaction, TxIn, TxOut};

use crate::{coin_selection::{CoinSelectionAlgorithm, DefaultCoinSelectionAlgorithm, Excess}, errors::{self, Error}, types::{self, Utxo}};
use wasi::random::random::{get_random_u64, get_random_bytes};

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

pub struct  WatchOnly {
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

impl WatchOnly {

    pub fn new(master_public: Xpub, network: Network) -> Self {
        WatchOnly {
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

    pub fn derive_p2wpkh_receive_address(& mut self) -> Result< AddressDetails ,errors::Error>{
        let secp = Secp256k1::new();
        let child_pub = self.master_public
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: 0 })
            .map_err(|err| errors::Error::PubKeyError(err) )?
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: self.receive_depth })
            .map_err(|err| errors::Error::PubKeyError(err) )?.to_pub();

        self.receive_depth +=1;    
        let pub_key = Address::p2wpkh(&child_pub, self.network)
            .script_pubkey();

        let hash =  pub_key.as_bytes().to_vec();

        return  Ok(AddressDetails { hash, human: pub_key.to_string() })
        
    }

    fn derive_p2wpkh_change_script(& mut self) -> Result< Vec<u8> ,errors::Error>{
        let secp = Secp256k1::new();
        let child_pub = self.master_public
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: 1 })
            .map_err(|err| errors::Error::PubKeyError(err) )?
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: self.change_depth })
            .map_err(|err| errors::Error::PubKeyError(err) )?.to_pub();

        self.change_depth +=1;    
        let pub_key = Address::p2wpkh(&child_pub, self.network)
            .script_pubkey();
            
        return  Ok(pub_key.to_bytes().to_vec())
        
    }

    fn derive_pubkey(&self, utxo: Utxo) -> Result<CompressedPublicKey, errors::Error> {
        let secp = Secp256k1::new();
        let child_pub = self.master_public
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: utxo.keychain.as_u32()})
            .map_err(|err| errors::Error::PubKeyError(err) )?
            .ckd_pub(&secp, bitcoin::bip32::ChildNumber::Normal { index: utxo.derivation_index })
            .map_err(|err| errors::Error::PubKeyError(err) )?.to_pub();

       
        Ok(child_pub)
    }

    pub fn create_psbt_tx(& mut self, recipient: Vec<u8>, fee_rate: FeeRate, amount: Amount) -> Result<Vec<u8>, errors::Error> {
        let change_script = self.derive_p2wpkh_change_script()?;
        let coinselection = DefaultCoinSelectionAlgorithm::default().coin_select(vec![], self.utxos.clone(), fee_rate, amount, Script::from_bytes(&change_script), &mut WasiRandom).map_err(|err| errors::Error::CoinSelection(err))?;
        
        let inputs = coinselection.selected.clone().iter().map(|utxo| TxIn {
            previous_output: utxo.outpoint,
            script_sig: Default::default(),
            sequence: Default::default(),
            witness: Default::default(),
        }).collect();

        let mut recipients = vec![TxOut {
            script_pubkey: ScriptBuf::from(recipient),
            value: amount,
        }];

        if let Excess::Change { amount, .. } = coinselection.excess {
            recipients.push(TxOut {
                script_pubkey: ScriptBuf::from(change_script),
                value: amount,
            });
        }

        let transaction = Transaction {
            version:  Version::TWO,
            lock_time: LockTime::ZERO,
            input: inputs,
            output: recipients,
        };

        let  mut psbt = Psbt::from_unsigned_tx(transaction).map_err(errors::Error::Psbt)?;

        let mut inputs=  Vec::new();
        let ty = PsbtSighashType::from(EcdsaSighashType::All);
        
        
        for utxo in coinselection.selected {
            let child_pub = self.derive_pubkey(utxo.clone())?;
            let mut map = BTreeMap::new();

            let derivation_path = DerivationPath::from(vec![ChildNumber::Normal{index: utxo.keychain.as_u32()}, ChildNumber::Normal{index: utxo.derivation_index }]);
            map.insert(child_pub.0, (self.master_public.parent_fingerprint, derivation_path ));

            let wpkh = child_pub.wpubkey_hash();
            let redeem_script = ScriptBuf::new_p2wpkh(&wpkh);

            let input = Input { witness_utxo: Some(utxo.txout) ,witness_script: Some(redeem_script),bip32_derivation: map, sighash_type: Some(ty),  ..Default::default()};
            inputs.push(input);
            
        };

        psbt.inputs = inputs;

        Ok(psbt.serialize())

    }
}



