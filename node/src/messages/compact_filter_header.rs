use crate::util::{sha256d, var_int, Error, Hash256, Result, Serializable};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::io;
use std::io::{Read, Write};
use crate::messages::message::Payload;


/// Block header
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct CompactFilterHeader {
    /// Block version specifying which validation rules to use
    pub filter_type: u8,
    /// Stop Hash
    pub stop_hash: Hash256,
    /// Previous Filter Header
    pub previous_filter_header: Hash256,
    /// Root of the merkle tree of this block's transaction hashes
    pub filter_hashes: Vec<Hash256>,
}

impl Payload<CompactFilterHeader> for CompactFilterHeader {
    fn size(&self) -> usize {
        1 + 32 + 32 + var_int::size(self.filter_hashes.len() as u64)
            + self.filter_hashes.len() * 32
    }
}

impl Serializable<CompactFilterHeader> for CompactFilterHeader {
    fn read(reader: &mut dyn Read) -> Result<CompactFilterHeader> {
        let filter_type = reader.read_u8()?;
        let stop_hash = Hash256::read(reader)?;
        let previous_filter_header = Hash256::read(reader)?;
        let filter_len = var_int::read(reader)?;
        let mut filter_hashes = Vec::new();
        for _i in 0..filter_len {
            filter_hashes.push(Hash256::read(reader)?);
        }
        Ok(CompactFilterHeader {
            filter_type,
            stop_hash,
            previous_filter_header,
            filter_hashes
        })
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u8(self.filter_type)?;
        self.stop_hash.write(writer)?;
        self.previous_filter_header.write(writer)?;
        var_int::write(self.filter_hashes.len() as u64, writer)?;
        for hash in self.filter_hashes.iter() {
            hash.write(writer)?;
        }
        Ok(())
    }
}