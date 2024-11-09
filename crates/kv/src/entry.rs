use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
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
    key: K,
    value: ValueReference,
    timestamp: u32,
    clock: Arc<dyn Clock>,
}

impl<K: BitCaskKey> Entry<K> {
    pub fn new(key: K, value: Vec<u8>, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value, tombstone: 0 },
            timestamp: 0,
            clock,
        }
    }

    pub fn new_preserving_timestamp(key: K, value: Vec<u8>, ts: u32, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value, tombstone: 0 },
            timestamp: ts,
            clock,
        }
    }

    pub fn new_deleted_entry(key: K, clock: Arc<dyn Clock>) -> Self {
        Entry {
            key,
            value: ValueReference { value: vec![], tombstone: 1 },
            timestamp: 0,
            clock,
        }
    }

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
        encoded.write_u32::<LittleEndian>(timestamp).unwrap();
        encoded.write_u32::<LittleEndian>(key_len_size).unwrap();
        encoded.write_u32::<LittleEndian>(value_len_size).unwrap();
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

pub fn decode(content: &[u8]) -> StoredEntry {
    decode_from(content, 0).0
}

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
            entry_length: traversed_offset,
        });
        offset = traversed_offset;
    }

    entries
}

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

#[derive(Clone)]

pub struct MappedStoredEntry<K> {
    pub key: K,
    pub value: Vec<u8>,
    pub deleted: bool,
    pub timestamp: u32,
    pub key_offset: u32,
    pub entry_length: u32,
}
