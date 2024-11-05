use std::fs;
use std::path::Path;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use crate::bit_cask_key::BitCaskKey;
use crate::config;
use crate::clock::Clock;
use crate::entry::{decode, decode_multi, Entry, MappedStoredEntry, StoredEntry};
use crate::errors::Error;
use crate::store::Store;

pub const SEGMENT_FILE_PREFIX: &str = "bitcask";
pub const SEGMENT_FILE_SUFFIX: &str = "data";

pub struct AppendEntryResponse {
    pub file_id: u64,
    pub offset: i64,
    pub entry_length: u32,
}
#[derive(Clone, Default)]
pub struct Segment<S: Store> {
    pub file_id: u64,
    file_path: String,
    store: S,
}

impl<S: Store> Segment<S> {
    pub fn new(file_id: u64, directory: &str) -> Result<Self, Error> {
        let file_path = segment_name(file_id);
        let store = S::new(&file_path, &directory)?;
        Ok(Segment {
            file_id,
            file_path,
            store,
        })
    }

    pub fn reload_inactive(file_id: u64, directory: &str) -> Result<Self, Error> {
        let file_path = segment_name(file_id);
        let store = S::new(&file_path, directory)?;
        Ok(Segment {
            file_id,
            file_path,
            store,
        })
    }

    pub fn append<K: BitCaskKey>(&mut self, entry: &Entry<K>) -> Result<AppendEntryResponse, Error> {
        let encoded = entry.encode();
        let offset = self.store.append(&encoded)?;
        Ok(AppendEntryResponse {
            file_id: self.file_id,
            offset,
            entry_length: encoded.len() as u32,
        })
    }

    pub fn read(&self, offset: i64, size: u32) -> Result<StoredEntry, Error> {
        let bytes = self.store.read(offset, size)?;
        Ok(decode(&bytes))
    }

    pub fn read_full<K: BitCaskKey>(&self, key_mapper: fn(&[u8]) -> K) -> Result<Vec<MappedStoredEntry<K>>, Error> {
        let bytes = self.store.read_full()?;
        Ok(decode_multi(&bytes, key_mapper))
    }

    pub fn size_in_bytes(&self) -> i64 {
        self.store.size_in_bytes()
    }

    pub fn sync(&self) {
        self.store.sync()
    }

    //TODO: FIX Please
    // pub fn stop_writes(&self) {
    //     self.store.stop_writes()
    // }

    pub fn remove(&mut self) {
        self.store.remove()
    }
}



fn segment_name(file_id: u64) -> String {
    let file_name = format!("{}_{}.{}", file_id, SEGMENT_FILE_PREFIX, SEGMENT_FILE_SUFFIX);
    return  file_name;
}
