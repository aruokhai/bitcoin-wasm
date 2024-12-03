use core::hash::Hash;
use std::str::FromStr;

use wasi::random;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait BitCaskKey: Serializable + PartialEq + Clone + Eq +  Hash {}


#[derive(Clone, Hash, PartialEq)]
pub struct UUIDWasiKey(uuid::Uuid);

impl Serializable for UUIDWasiKey {
    fn serialize(&self) -> Vec<u8> {
        return  self.0.as_bytes().to_vec();
    }
}

impl Eq for UUIDWasiKey {}
impl BitCaskKey for UUIDWasiKey {}



impl UUIDWasiKey {
    fn new(&self) -> Self {
        UUIDWasiKey(uuid::Uuid::from_u64_pair(random::random::get_random_u64(), random::random::get_random_u64()))
    }

}

impl From<String> for UUIDWasiKey {
    fn from(value: String) -> Self {
        UUIDWasiKey(uuid::Uuid::from_str(&value).unwrap())
    }
}

pub fn UUIDWasiKeyFrom(value: &[u8]) -> UUIDWasiKey {
        UUIDWasiKey(uuid::Uuid::from_slice(&value).unwrap())
}

