use std::collections::HashMap;
use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::error::Error;
use std::fmt;

use crate::bit_cask_key::BitCaskKey;
use crate::key_directory::KeyDirectory;
use crate::segments::Segments;
use crate::store::Store;


pub struct KVStore<Key: BitCaskKey, S: Store> {
    segments: Segments<S>,
    key_directory: KeyDirectory<Key>,
    lock: RwLock<()>,
}

impl<Key: BitCaskKey> KVStore<Key> {
    pub fn new(config: &Config<Key>) -> Result<Self, Box<dyn Error>> {
        let segments = Segments::new(config.directory(), config.max_segment_size_in_bytes(), config.clock())?;
        let mut store = KVStore {
            segments,
            key_directory: KeyDirectory::new(config.key_directory_capacity()),
            lock: RwLock::new(()),
        };
        store.reload(config)?;
        Ok(store)
    }

    pub fn put(&self, key: Key, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let _write_lock = self.lock.write().unwrap();
        let append_entry_response = self.segments.append(key.clone(), value)?;
        self.key_directory.put(key, Entry::from(append_entry_response));
        Ok(())
    }

    pub fn update(&self, key: Key, value: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.put(key, value)
    }

    pub fn delete(&self, key: Key) -> Result<(), Box<dyn Error>> {
        let _write_lock = self.lock.write().unwrap();
        self.segments.append_deleted(key.clone())?;
        self.key_directory.delete(&key);
        Ok(())
    }

    pub fn silent_get(&self, key: Key) -> Option<Vec<u8>> {
        let _read_lock = self.lock.read().unwrap();
        if let Some(entry) = self.key_directory.get(&key) {
            if let Ok(stored_entry) = self.segments.read(entry.file_id, entry.offset, entry.entry_length) {
                return Some(stored_entry.value);
            }
        }
        None
    }

    pub fn get(&self, key: Key) -> Result<Vec<u8>, Box<dyn Error>> {
        let _read_lock = self.lock.read().unwrap();
        if let Some(entry) = self.key_directory.get(&key) {
            let stored_entry = self.segments.read(entry.file_id, entry.offset, entry.entry_length)?;
            return Ok(stored_entry.value);
        }
        Err(Box::new(fmt::Error))
    }

    pub fn write_back(&self, file_ids: Vec<u64>, changes: HashMap<Key, MappedStoredEntry<Key>>) -> Result<(), Box<dyn Error>> {
        let _write_lock = self.lock.write().unwrap();
        let write_back_responses = self.segments.write_back(changes)?;
        self.key_directory.bulk_update(write_back_responses);
        self.segments.remove(file_ids);
        Ok(())
    }

    pub fn clear_log(&self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.remove_active();
        self.segments.remove_all_inactive();
    }

    pub fn sync(&self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.sync();
    }

    pub fn shutdown(&self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.shutdown();
    }

    pub fn reload(&mut self, config: &Config<Key>) -> Result<(), Box<dyn Error>> {
        let _write_lock = self.lock.write().unwrap();
        for (file_id, segment) in self.segments.all_inactive_segments() {
            let entries = segment.read_full(config.merge_config().key_mapper())?;
            self.key_directory.reload(file_id, entries);
        }
        Ok(())
    }
}
