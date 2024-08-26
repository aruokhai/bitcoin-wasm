use crate::messages::OutPoint;
use crate::util::{var_int, Result, Serializable};
use bitcoin::Witness;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{io, vec};
use std::io::{Read, Write};

/// Transaction input
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct TxIn {
    /// The previous output transaction reference
    pub prev_output: OutPoint,
    /// Signature script for confirming authorization
    pub unlock_script: Vec<u8>,
    /// Transaction version as defined by the sender for replacement or negotiation
    pub sequence: u32,

}


impl Serializable<TxIn> for TxIn {
    fn read(reader: &mut dyn Read) -> Result<TxIn> {
        let prev_output = OutPoint::read(reader)?;
        let script_len = var_int::read(reader)?;
        let mut unlock_script = Vec::new();
        for _i in 0..script_len {
            unlock_script.push(reader.read_u8()?)
        }
        let sequence = reader.read_u32::<LittleEndian>()?;
        Ok(TxIn {
            prev_output,
            unlock_script,
            sequence,
        })
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        self.prev_output.write(writer)?;
        var_int::write(self.unlock_script.len() as u64, writer)?;
        for bytes in self.unlock_script.iter() {
            writer.write_u8(*bytes)?;
        }
        writer.write_u32::<LittleEndian>(self.sequence)?;
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::util::Hash256;
//     use std::io::Cursor;

//     #[test]
//     fn write_read() {
//         let mut v = Vec::new();
//         let t = TxIn {
//             prev_output: OutPoint {
//                 hash: Hash256([6; 32]),
//                 index: 8,
//             },
//             unlock_script: Script(vec![255; 254]),
//             sequence: 100,
//         };
//         t.write(&mut v).unwrap();
//         assert!(v.len() == t.size());
//         assert!(TxIn::read(&mut Cursor::new(&v)).unwrap() == t);
//     }
// }
