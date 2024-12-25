use std::collections::HashMap;

use crate::bit_cask_key::BitCaskKey;
use crate::entry::MappedStoredEntry;
use crate::segment::AppendEntryResponse;
use crate::segments::WriteBackResponse;

/// KeyDirectory is the in-memory storage which maintains a mapping between keys and the position of those keys in the datafiles called segment.
/// Entry maintains `FileId` identifying the file containing the key, `Offset` identifying the position in the file where the key is stored and
/// the `EntryLength` identifying the length of the entry
pub struct KeyDirectory<Key: BitCaskKey> {
    entry_by_key: HashMap<Key, Entry>,
}

impl<Key: BitCaskKey> KeyDirectory<Key> {
    pub fn new(initial_capacity: usize) -> Self {
        KeyDirectory {
            entry_by_key: HashMap::with_capacity(initial_capacity),
        }
    }

    // Reload reloads the state of the KeyDirectory during start-up. As a part of reloading the state in bitcask model, all the inactive segments are read,
    // and the keys from all the inactive segments are stored in the KeyDirectory.
    // Riak's paper optimizes reloading by creating small sized hint files during merge and compaction.
    // Hint files contain the keys and the metadata fields like fileId, fileOffset and entryLength, these hint files are referred during reload. This implementation does not create Hint file
    pub fn reload(&mut self, file_id: u64, entries: Vec<MappedStoredEntry<Key>>) {
        for entry in entries {
            self.entry_by_key.insert(
                entry.key.clone(),
                Entry::new(file_id, entry.key_offset as i64, entry.entry_length),
            );
        }
    }

    /// Put puts a key and its entry as the value in the KeyDirectory
    pub fn put(&mut self, key: Key, value: Entry) {
        self.entry_by_key.insert(key, value);
    }

    /// BulkUpdate performs bulk changes to the KeyDirectory state. This method is called during merge and compaction from KeyStore.
    pub fn bulk_update(&mut self, changes: Vec<WriteBackResponse<Key>>) {
        for change in changes {
            self.entry_by_key.insert(
                change.key.clone(),
                Entry::from(change.append_entry_response),
            );
        }
    }

    /// Delete removes the key from the KeyDirectory
    pub fn delete(&mut self, key: &Key) {
        self.entry_by_key.remove(key);
    }

    // Get returns the Entry and a boolean to indicate if the value corresponding to the key is present in the KeyDirectory.
    pub fn get(&self, key: &Key) -> Option<&Entry> {
        self.entry_by_key.get(key)
    }
}

pub struct Entry {
    pub file_id: u64,
    pub offset: i64,
    pub entry_length: u32,
}

impl Entry {
    // Entry (pointer to the Entry) is used as a value in the KeyDirectory
    // It identifies the file containing the key, the offset of the key-value in the file and the entry length.
    // Refer to Entry.go inside log/ package to understand encoding and decoding.
    pub fn new(file_id: u64, offset: i64, entry_length: u32) -> Self {
        Self {
            file_id,
            offset,
            entry_length,
        }
    }
}

impl From<AppendEntryResponse> for Entry {
    fn from(response: AppendEntryResponse) -> Self {
        Self {
            file_id: response.file_id,
            offset: response.offset,
            entry_length: response.entry_length,
        }
    }
}