use core::hash::Hash;
use std::str::FromStr;

use wasi::random;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait BitCaskKey: Serializable + PartialEq + Clone + Eq +  Hash {}


#[derive(Clone, Hash, PartialEq)]
pub struct UUIDWasiKey(String);

impl Serializable for UUIDWasiKey {
    fn serialize(&self) -> Vec<u8> {
        return  self.0.as_bytes().to_vec();
    }
}

impl Eq for UUIDWasiKey {}
impl BitCaskKey for UUIDWasiKey {}





impl From<String> for UUIDWasiKey {
    fn from(value: String) -> Self {
        UUIDWasiKey(value)
    }
}

pub fn UUIDWasiKeyFrom(value: &[u8]) -> UUIDWasiKey {
        UUIDWasiKey(String::from_utf8(value.to_vec()).unwrap())
}

