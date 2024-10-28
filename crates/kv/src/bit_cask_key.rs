pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait BitCaskKey: Serializable + PartialEq {}