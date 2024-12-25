use crate::messages::message::Payload;
use crate::util::{Hash256, Result, Serializable};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Read, Write};

/// Return results until either there are 2000 for getheaders or 500 or getblocks, or no more left
pub const NO_HASH_STOP: Hash256 = Hash256([0; 32]);

/// Specifies which blocks to return
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct FilterLocator {
    /// Protocol version of this node
    pub filter_type: u8,
    /// Block hash to start after. First found will be used.
    pub start_height: u32,
    /// Block hash to stop at, or none if NO_HASH_STOP.
    pub hash_stop: Hash256,
}


impl Serializable<FilterLocator> for FilterLocator {
    fn read(reader: &mut dyn Read) -> Result<FilterLocator> {
        let filter_type = reader.read_u8()?;
        let start_height = reader.read_u32::<LittleEndian>()?;
        let hash_stop = Hash256::read(reader)?;
        Ok(FilterLocator {
            filter_type,
            start_height,
            hash_stop,
        })
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u8(self.filter_type)?;
        writer.write_u32::<LittleEndian>(self.start_height)?;
        self.hash_stop.write(writer)?;
        Ok(())
    }
}

impl Payload<FilterLocator> for FilterLocator {
    fn size(&self) -> usize {
        1 + 4 + 32
    }
}

#[cfg(test)]
mod tests {
    use crate::messages::block_locator::BlockLocator;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn write_read() {
        let mut v = Vec::new();
        let p = BlockLocator {
            version: 12345,
            block_locator_hashes: vec![
                NO_HASH_STOP,
                Hash256::decode("6677889900667788990066778899006677889900667788990066778899006677")
                    .unwrap(),
            ],
            hash_stop: Hash256::decode(
                "1122334455112233445511223344551122334455112233445511223344551122",
            )
            .unwrap(),
        };
        p.write(&mut v).unwrap();
        assert!(v.len() == p.size());
        assert!(BlockLocator::read(&mut Cursor::new(&v)).unwrap() == p);
    }
}
