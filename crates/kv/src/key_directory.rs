use std::collections::HashMap;
use std::hash::Hash;

use crate::bit_cask_key::BitCaskKey;
use crate::entry::{ MappedStoredEntry};
use crate::segment::AppendEntryResponse;
use crate::segments::WriteBackResponse;


pub struct Entry {
    file_id: u64,
    offset: i64,
    entry_length: u32,
}

impl Entry {

    // Equivalent to Go's NewEntry function
    pub fn new(file_id: u64, offset: i64, entry_length: u32) -> Self {
        Self {
            file_id,
            offset,
            entry_length,
        }
    }
}

// Implement the From trait for converting from AppendEntryResponse to Entry
impl From<AppendEntryResponse> for Entry {
    fn from(response: AppendEntryResponse) -> Self {
        Self {
            file_id: response.file_id,
            offset: response.offset,
            entry_length: response.entry_length,
        }
    }
}

pub struct KeyDirectory<Key: BitCaskKey> {
    entry_by_key: HashMap<Key, Entry>,
}

impl<Key: BitCaskKey + Clone + Eq +  Hash> KeyDirectory<Key> {
    // NewKeyDirectory equivalent constructor function
    pub fn new(initial_capacity: usize) -> Self {
        KeyDirectory {
            entry_by_key: HashMap::with_capacity(initial_capacity),
        }
    }

    // Reload equivalent method
    pub fn reload(&mut self, file_id: u64, entries: Vec<MappedStoredEntry<Key>>) {
        for entry in entries {
            self.entry_by_key.insert(
                entry.key.clone(),
                Entry::new(file_id, entry.key_offset as i64, entry.entry_length),
            );
        }
    }

    // Put equivalent method
    pub fn put(&mut self, key: Key, value: Entry) {
        self.entry_by_key.insert(key, value);
    }

    // BulkUpdate equivalent method
    pub fn bulk_update(&mut self, changes: Vec<WriteBackResponse<Key>>) {
        for change in changes {
            self.entry_by_key.insert(
                change.key.clone(),
                Entry::from(change.append_entry_response),
            );
        }
    }

    // Delete equivalent method
    pub fn delete(&mut self, key: &Key) {
        self.entry_by_key.remove(key);
    }

    // Get equivalent method
    pub fn get(&self, key: &Key) -> Option<&Entry> {
        self.entry_by_key.get(key)
    }
}