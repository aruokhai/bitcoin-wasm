use std::collections::HashMap;
use std::sync::{RwLock};

use crate::bit_cask_key::BitCaskKey;
use crate::config::Config;
use crate::entry::{MappedStoredEntry};
use crate::errors::Error;
use crate::key_directory::{ KeyDirectory, Entry as KeyDirectoryEntry };
use crate::merge_config::MergeConfig;
use crate::merged_state::MergedState;
use crate::segments::Segments;
use crate::store::Store;

/// KVStore encapsulates append-only log segments and KeyDirectory which is an in-memory hashmap
/// Segments is an abstraction that manages the active and K inactive segments.
/// KVStore also maintains a RWLock that allows an exclusive writer and N readers
pub struct KVStore<Key: BitCaskKey, S: Store> {
    segments: Segments<S>,
    key_directory: KeyDirectory<Key>,
    lock: RwLock<()>,
    merge_config: MergeConfig<Key>,
    counter: u64,
}


impl<Key: BitCaskKey, S: Store > KVStore<Key, S> {
    /// It creates a new instance of KVStore
    /// It also performs a reload operation `store.reload(config)` that is responsible for reloading the state of KeyDirectory from inactive segments
    pub fn new(config: &Config<Key>) -> Result<Self, Error> {
        let segments = Segments::new::<Key>(config.directory().into(), config.max_segment_size_in_bytes(), config.clock())?;
        let mut store = KVStore {
            segments,
            key_directory: KeyDirectory::new(config.key_directory_capacity() as usize),
            lock: RwLock::new(()),
            merge_config: config.merge_config().unwrap().clone(),
            counter: 0
        };
        store.reload()?;
        Ok(store)
    }

    /// Put puts the key and the value in bitcask. Put operations consists of the following steps:
    /// 1. Perform Merge if counter is reached
    /// 2.Append the key and the value in the append-only active segment using `kv.segments.Append(key, value)`.
    /// - Segments abstraction will append the key and the value to the active segment if the size of the active segment is less than the threshold, else it will perform a rollover of the active segment
    /// 3.Once the append operation is successful, it will write the key and the Entry to the KeyDirectory, which is an in-memory representation of the key and its position in an append-only segment
    fn put(&mut self, key: Key, value: Vec<u8>) -> Result<(), Error> {
        if self.counter >= self.merge_config.run_merge_every() {
            self.begin_merge()?;
            self.counter = 0
        }

        let _write_lock = self.lock.write().unwrap();
        let append_entry_response = self.segments.append(key.clone(), value)?;
        self.key_directory.put(key, KeyDirectoryEntry::from(append_entry_response));
        self.counter +=1;
        Ok(())
    }

    /// Update is very much similar to Put. It appends the key and the value to the log and performs an in-place update in the KeyDirectory
    pub fn update(&mut self, key: Key, value: Vec<u8>) -> Result<(), Error> {
        self.put(key, value)
    }

    /// Delete appends the key and the value to the log and performs an in-place delete in the KeyDirectory
    pub fn delete(& mut self, key: Key) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        self.segments.append_deleted(key.clone())?;
        self.key_directory.delete(&key);
        Ok(())
    }

    /// Get gets the value corresponding to the key. Returns value and nil if the value is found, else returns nil and error
    /// In order to perform Get, a Get operation is performed in the KeyDirectory which returns an Entry indicating the fileId, offset of the key and the entry length
    /// If an Entry corresponding to the key is found, a Read operation is performed in the Segments abstraction, which performs an in-memory lookup to identify the segment based on the fileId, and then a Read operation is performed in that Segment
    pub fn get(&self, key: Key) -> Result<Vec<u8>, Error> {
        let _read_lock = self.lock.read().unwrap();
        if let Some(entry) = self.key_directory.get(&key) {
            let stored_entry = self.segments.read(entry.file_id, entry.offset, entry.entry_length)?;
            return Ok(stored_entry.value);
        }
        Err(Error::EntryNotFound)
    }

    /// WriteBack writes back the changes (merged changes) to new inactive segments. This operation is performed during merge.
    /// It writes all the changes into M new inactive segments and once those changes are written to the new inactive segment(s), the state of the keys present in the `changes` parameter is updated in the KeyDirectory. More on this is mentioned in Worker.go inside merge/ package.
    /// Once the state is updated in the KeyDirectory, the old segments identified by `fileIds` are removed from disk.
    fn write_back(&mut self, file_ids: Vec<u64>, changes: HashMap<Key, MappedStoredEntry<Key>>) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        let write_back_responses = self.segments.write_back(changes)?;
        self.key_directory.bulk_update(write_back_responses);
        self.segments.remove(file_ids.as_slice());
        Ok(())
    }

    // ClearLog removes all the log files
    pub fn clear_log(&mut self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.remove_active();
        self.segments.remove_all_inactive();
    }

    // Sync performs a sync of all the active and inactive segments. 
    fn sync(&self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.sync();
    }

    // reload the entire state during start-up.
    fn reload(&mut self) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        for (file_id, segment) in self.segments.all_inactive_segments() {
            let entries = segment.read_full(self.merge_config.key_mapper())?;
            self.key_directory.reload(*file_id, entries);
        }
        Ok(())
    }

    /// beginMerge performs the merge operation.
    /// As a part of merge process, either all the inactive segments files are read or any of the K inactive segment files are read in memory.
    /// Once those files are loaded in memory, an instance of MergedState is created that maintains a HashMap of Key and MappedStoredEntry.
    /// MergedState is responsible for performing the merge operation. Merge operation is all about picking the latest value of a key
    /// if it is present in 2 or more segment files.
    /// Once the merge operation is done, the changes are written back to new inactive files and the in-memory state is updated in KeyDirectory.
    ///
    /// Why do we need to update the in-memory state?
    /// Assume a Key K1 with Value V1 and Timestamp T1 is present in the segment file F1. This key gets updated with value V2 at a later timestamp T2
    /// and these changes were written to a new active segment file F2. At some point in time, F2 becomes inactive.
    /// At this stage the KeyDirectory will contain the following mapping for the key K1, <K1 => {F2, Offset, EntryLength}>.
    ///  Segment file F1 ```
    ///	┌───────────┬──────────┬────────────┬─────┬───────┐
    ///	│ T1        │ key_size │ value_size │ K1  │ V1    │
    ///	└───────────┴──────────┴────────────┴─────┴───────┘
    /// ```
    ///  Segment file F2 ```
    ///	┌───────────┬──────────┬────────────┬─────┬───────┐
    ///	│ T2        │ key_size │ value_size │ K1  │ V2    │
    ///	└───────────┴──────────┴────────────┴─────┴───────┘
    /// ```
    /// KeyDirectory contains K1 pointing to the offset of K1 in the segment file F2.
    /// With this background, let's consider that the merge process starts, and it reads the contents of F1 and F2 and performs a merge.
    /// The merge writes the key K1 with its new value V2 and timestamp T2 in a new file F3, and deletes files F1 and F2.
    ///
    ///	 Segment file F3 ```
    ///	┌───────────┬──────────┬────────────┬─────┬───────┐
    ///	│ T2        │ key_size │ value_size │ K1  │ V2    │
    ///	└───────────┴──────────┴────────────┴─────┴───────┘
    /// ```
    /// The moment merge process is done, the state of Key K1 needs to be updated in the KeyDirectory to point to the new offset in the new file.
    fn begin_merge(&mut self) -> Result<(), Error> {
        let (file_ids, segments)  = 
            if self.merge_config.should_read_all_segments() {
                self.segments.read_all_inactive_segments(self.merge_config.key_mapper())?
            } else {
                self.segments.read_inactive_segments(self.merge_config.total_segments_to_read(), self.merge_config.key_mapper())?
            };

        if segments.len() >= 2 {
            let mut merged_state = MergedState::new();
            merged_state.take_all(segments[0].to_owned());

            for segment in &segments[1..] {
                merged_state.merge_with(segment.to_owned());
            }

            // Ignoring the result for `write_back`, as in the Go code
            let _ = self.write_back(file_ids, merged_state.value_by_key.clone());
        }

        self.sync();

        Ok(())
    }
}
