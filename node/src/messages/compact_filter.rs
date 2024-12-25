use crate::util::{var_int, Hash256, Result, Serializable};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Read, Write};
use crate::messages::message::Payload;


/// Block header
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct CompactFilter {
    /// Block version specifying which validation rules to use
    pub filter_type: u8,
    /// Hash of the previous block
    pub block_hash: Hash256,
    /// Root of the merkle tree of this block's transaction hashes
    pub filter_bytes: Vec<u8>,
}

impl Payload<CompactFilter> for CompactFilter {
    fn size(&self) -> usize {
        1 + 32 + var_int::size(self.filter_bytes.len() as u64)
            + self.filter_bytes.len()
    }
}

impl Serializable<CompactFilter> for CompactFilter {
    fn read(reader: &mut dyn Read) -> Result<CompactFilter> {
        let filter_type = reader.read_u8()?;
        let block_hash = Hash256::read(reader)?;
        let filter_len = var_int::read(reader)?;
        let mut filter_bytes = Vec::new();
        for _i in 0..filter_len {
            filter_bytes.push(reader.read_u8()?);
        }
        Ok(CompactFilter {
            filter_type,
            filter_bytes,
            block_hash,
        })
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u8(self.filter_type)?;
        self.block_hash.write(writer)?;
        var_int::write(self.filter_bytes.len() as u64, writer)?;
        for byte in self.filter_bytes.iter() {
            writer.write_u8(*byte)?;
        }
        Ok(())
    }
}