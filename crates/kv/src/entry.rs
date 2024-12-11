use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use byteorder::{ByteOrder, LittleEndian};
use crate::config;
use crate::clock::Clock;
use crate::bit_cask_key::Serializable;
use crate::bit_cask_key::BitCaskKey;

const RESERVED_KEY_SIZE: u32 = mem::size_of::<u32>() as u32;
const RESERVED_VALUE_SIZE: u32 = mem::size_of::<u32>() as u32;
const RESERVED_TIMESTAMP_SIZE: u32 = mem::size_of::<u32>() as u32;
const TOMBSTONE_MARKER_SIZE: u32 = mem::size_of::<u8>() as u32;

#[derive(Clone)]
struct ValueReference {
    value: Vec<u8>,
    tombstone: u8,
}

#[derive(Clone)]

pub struct Entry<K: BitCaskKey> {
    pub key: K,
    value: ValueReference,
    timestamp: u32,
    clock: Arc<dyn Clock>,
}

impl<K: BitCaskKey> Entry<K> {
    /// NewEntry creates a new instance of Entry with tombstone byte set to 0 (0000 0000)
    pub fn new(key: K, value: Vec<u8>, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value, tombstone: 0 },
            timestamp: 0,
            clock,
        }
    }
    /// NewEntryPreservingTimestamp creates a new instance of Entry with tombstone byte set to 0 (0000 0000) and keeping the provided timestamp
    pub fn new_preserving_timestamp(key: K, value: Vec<u8>, ts: u32, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value, tombstone: 0 },
            timestamp: ts,
            clock,
        }
    }

    /// NewDeletedEntry creates a new instance of Entry with tombstone byte set to 1 (0000 0001)
    pub fn new_deleted_entry(key: K, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value: vec![], tombstone: 1 },
            timestamp: 0,
            clock,
        }
    }

    /// encode performs the encode operation which converts the Entry to a byte slice which can be written to the disk
    /// Encoding scheme consists of the following structure:
    /// ```
    ///	┌───────────┬──────────┬────────────┬─────┬───────┐
    ///	│ timestamp │ key_size │ value_size │ key │ value │
    ///	└───────────┴──────────┴────────────┴─────┴───────┘
    /// ```
    /// timestamp, key_size, value_size consist of 32 bits each. The value ([]byte) consists of the value provided by the user and a byte for tombstone, that
    /// is used to signify if the key/value pair is deleted or not. Take a look at the NewDeletedEntry function.
    /// A little-endian system, stores the least-significant byte at the smallest address. What is special about 4 bytes key size or 4 bytes value size?
    /// The maximum integer stored by 4 bytes is 4,294,967,295 (2 ** 32 - 1), roughly ~4.2GB. This means each key or value size can not be greater than 4.2GB.
    pub fn encode(&self) -> Vec<u8> {
        let serialized_key = self.key.serialize();
        let key_len_size = serialized_key.len() as u32;
        let value_len_size = self.value.value.len() as u32 + TOMBSTONE_MARKER_SIZE;

        let mut encoded = Vec::with_capacity(
            (RESERVED_TIMESTAMP_SIZE + RESERVED_KEY_SIZE + RESERVED_VALUE_SIZE + key_len_size + value_len_size) as usize,
        );

        let timestamp = if self.timestamp == 0 {
            self.clock.now() as u32
        } else {
            self.timestamp
        };

        // Write the header
        encoded.extend_from_slice(&timestamp.to_le_bytes()); // Write timestamp as little-endian
        encoded.extend_from_slice(&key_len_size.to_le_bytes());   // Write key length
        encoded.extend_from_slice(&value_len_size.to_le_bytes()); // Write value length
        encoded.extend_from_slice(&serialized_key);
        encoded.extend_from_slice(&self.value.value);
        encoded.push(self.value.tombstone);

        encoded
    }
}

pub struct StoredEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub deleted: bool,
    pub timestamp: u32,
}

/// decode performs the decode operation and returns an instance of StoredEntry
pub fn decode(content: &[u8]) -> StoredEntry {
    decode_from(content, 0).0
}

/// decodeMulti performs multiple decode operations and returns an array of MappedStoredEntry
/// This method is invoked when a segment file needs to be read completely. This happens during reload and merge operations.
pub fn decode_multi<K: BitCaskKey>(
    content: &[u8],
    key_mapper: fn(&[u8]) -> K,
) -> Vec<MappedStoredEntry<K>> {
    let content_length = content.len() as u32;
    let mut offset = 0;
    let mut entries = Vec::new();

    while offset < content_length {
        let (entry, traversed_offset) = decode_from(content, offset);
        entries.push(MappedStoredEntry {
            key: key_mapper(&entry.key),
            value: entry.value.clone(),
            deleted: entry.deleted,
            timestamp: entry.timestamp,
            key_offset: offset,
            // TODO: verify if correct
            entry_length: traversed_offset - offset,
        });
        offset = traversed_offset;
    }

    entries
}

/// decodeFrom performs the decode operation.
/// Encoding scheme consists of the following structure:
/// ```
///	┌───────────┬──────────┬────────────┬─────┬───────┐
///	│ timestamp │ key_size │ value_size │ key │ value │
///	└───────────┴──────────┴────────────┴─────┴───────┘
/// ```
/// In order to perform `decode`, the code reads the first 4 bytes to get the timestamp, next 4 bytes to get the key size, next 4 bytes to get the value size
/// Note: the value size is the size including the length of the byte slice provided by the user and one byte for the tombstone marker
/// Reading further from the offset to the offset+keySize return the actual key, followed by next read from offset to offset+valueSize which returns the actual value.
/// DeletedFlag is determined by taking the last byte from the `value` byte slice and performing an AND operation with 0x01.
fn decode_from(content: &[u8], mut offset: u32) -> (StoredEntry, u32) {
    let timestamp = LittleEndian::read_u32(&content[offset as usize..]);
    offset += RESERVED_TIMESTAMP_SIZE;

    let key_size = LittleEndian::read_u32(&content[offset as usize..]);
    offset += RESERVED_KEY_SIZE;

    let value_size = LittleEndian::read_u32(&content[offset as usize..]);
    offset += RESERVED_VALUE_SIZE;

    let serialized_key = &content[offset as usize..(offset + key_size) as usize];
    offset += key_size;

    let value = &content[offset as usize..(offset + value_size) as usize];
    offset += value_size;

    let value_length = value.len();
    (
        StoredEntry {
            key: serialized_key.to_vec(),
            value: value[..value_length - 1].to_vec(),
            deleted: (value[value_length - 1] & 0x01) == 0x01,
            timestamp,
        },
        offset,
    )
}

#[derive(Clone, Debug)]
pub struct MappedStoredEntry<K> {
    pub key: K,
    pub value: Vec<u8>,
    pub deleted: bool,
    pub timestamp: u32,
    pub key_offset: u32,
    pub entry_length: u32,
}
