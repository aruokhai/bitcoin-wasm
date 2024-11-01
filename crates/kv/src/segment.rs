use std::fs;
use std::path::Path;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use crate::bit_cask_key::BitCaskKey;
use crate::config;
use crate::clock::Clock;
use crate::store::Store;

pub struct StoredEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub deleted: bool,
    pub timestamp: u32,
}

pub struct MappedStoredEntry<K> {
    pub key: K,
    pub value: Vec<u8>,
    pub deleted: bool,
    pub timestamp: u32,
    pub key_offset: u32,
    pub entry_length: u32,
}

pub struct AppendEntryResponse {
    pub file_id: u64,
    pub offset: i64,
    pub entry_length: u32,
}

pub struct Segment<K: BitCaskKey> {
    file_id: u64,
    file_path: String,
    store: Box< dyn Store>,
}

impl<K: BitCaskKey> Segment<K> {
    pub fn new(file_id: u64, directory: &str) -> Result<Self, std::io::Error> {
        let file_path = create_segment(file_id, directory)?;
        let store = <dyn Store>::new(&file_path)?;
        Ok(Segment {
            file_id,
            file_path,
            store,
        })
    }

    pub fn reload_inactive(file_id: u64, directory: &str) -> Result<Self, std::io::Error> {
        let file_path = segment_name(file_id, directory);
        let store = <dyn Store>::reload(&file_path)?;
        Ok(Segment {
            file_id,
            file_path,
            store,
        })
    }

    pub fn append(&mut self, entry: &Entry<K>) -> Result<AppendEntryResponse, std::io::Error> {
        let encoded = entry.encode();
        let offset = self.store.append(&encoded)?;
        Ok(AppendEntryResponse {
            file_id: self.file_id,
            offset,
            entry_length: encoded.len() as u32,
        })
    }

    pub fn read(&self, offset: i64, size: u32) -> Result<StoredEntry, std::io::Error> {
        let bytes = self.store.read(offset, size)?;
        Ok(decode(&bytes))
    }

    pub fn read_full(&self, key_mapper: fn(&[u8]) -> K) -> Result<Vec<MappedStoredEntry<K>>, std::io::Error> {
        let bytes = self.store.read_full()?;
        Ok(decode_multi(&bytes, key_mapper))
    }

    pub fn size_in_bytes(&self) -> i64 {
        self.store.size_in_bytes()
    }

    pub fn sync(&self) {
        self.store.sync()
    }

    pub fn stop_writes(&self) {
        self.store.stop_writes()
    }

    pub fn remove(&self) {
        self.store.remove()
    }
}

fn create_segment(file_id: u64, directory: &str) -> Result<String, std::io::Error> {
    let file_path = segment_name(file_id, directory);
    fs::File::create(&file_path)?;
    Ok(file_path)
}

fn segment_name(file_id: u64, directory: &str) -> String {
    let file_name = format!("{}_{}.{}", file_id, SEGMENT_FILE_PREFIX, SEGMENT_FILE_SUFFIX);
    Path::new(directory).join(file_name).to_str().unwrap().to_string()
}
