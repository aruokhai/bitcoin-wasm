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

//! Errors that can be thrown by the [`Wallet`](crate::wallet::Wallet)


use crate::coin_selection;
use bitcoin::{absolute, psbt, Amount, OutPoint, Sequence, Txid, bip32::Error as Bip32_Error};
use core::fmt;



#[derive(Debug)]
/// Error returned from [`TxBuilder::finish`]
///
/// [`TxBuilder::finish`]: crate::wallet::tx_builder::TxBuilder::finish
pub enum Error {
    /// There was an error with coin selection
    CoinSelection(coin_selection::InsufficientFunds),

    /// Partially signed bitcoin transaction error
    Psbt(psbt::Error),

    /// Missing non_witness_utxo on foreign utxo for given `OutPoint`
    MissingNonWitnessUtxo(OutPoint),
    /// Creating Pubkey Error
    PubKeyError(Bip32_Error),
    NoPubKey,
    
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
        
            Error::CoinSelection(e) => e.fmt(f),
 
            Error::Psbt(e) => e.fmt(f),

            Error::MissingNonWitnessUtxo(outpoint) => {
                write!(f, "Missing non_witness_utxo on foreign utxo {}", outpoint)
            }
            Error::PubKeyError(error) => error.fmt(f),
            Error::NoPubKey => write!(f, "Cannot find PubKey")
        }
    }
}



impl From<psbt::Error> for Error {
    fn from(err: psbt::Error) -> Self {
        Error::Psbt(err)
    }
}

impl From<coin_selection::InsufficientFunds> for Error {
    fn from(err: coin_selection::InsufficientFunds) -> Self {
        Error::CoinSelection(err)
    }
}
