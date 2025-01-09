// Bitcoin Dev Kit
// Written in 2020 by Alekos Filini <alekos.filini@gmail.com>
//
// Copyright (c) 2020-2021 Bitcoin Dev Kit Developers
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use core::convert::AsRef;

use bitcoin::transaction::{OutPoint, Sequence, TxOut};
use bitcoin::{psbt, Weight};

use serde::{Deserialize, Serialize};

/// Types of keychains
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum KeychainKind {
    /// External keychain, used for deriving recipient addresses.
    External = 0,
    /// Internal keychain, used for deriving change addresses.
    Internal = 1,
}

impl KeychainKind {
    /// Return [`KeychainKind`] as a byte
    pub fn as_u32(&self) -> u32 {
        match self {
            KeychainKind::External =>  0,
            KeychainKind::Internal => 1,
        }
    }
}

impl AsRef<[u8]> for KeychainKind {
    fn as_ref(&self) -> &[u8] {
        match self {
            KeychainKind::External => b"e",
            KeychainKind::Internal => b"i",
        }
    }
}



/// A [`Utxo`] with its `satisfaction_weight`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedUtxo {
    /// The weight of the witness data and `scriptSig` expressed in [weight units]. This is used to
    /// properly maintain the feerate when adding this input to a transaction during coin selection.
    ///
    /// [weight units]: https://en.bitcoin.it/wiki/Weight_units
    pub satisfaction_weight: Weight,
    /// The UTXO
    pub utxo: Utxo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unspent transaction output (UTXO).
pub struct Utxo {
   /// Reference to a transaction output
   pub outpoint: OutPoint,
   /// Transaction output
   pub txout: TxOut,
   /// Type of keychain
   pub keychain: KeychainKind,
   /// Whether this UTXO is spent or not
   pub is_spent: bool,
   /// The derivation index for the script pubkey in the wallet
   pub derivation_index: u32,
   /// The position of the output in the blockchain.
   pub chain_position: Option<u32>,
    
}

impl Utxo {
    /// Get the location of the UTXO
    pub fn outpoint(&self) -> OutPoint {

        return  self.outpoint;
    }

    /// Get the `TxOut` of the UTXO
    pub fn txout(&self) -> &TxOut {
        return &self.txout
    }
}
