
pub struct Wallet {
    pub address: String,
    pub p2wkh_script: Vec<u8>,
    pub amount_sats: i64
}