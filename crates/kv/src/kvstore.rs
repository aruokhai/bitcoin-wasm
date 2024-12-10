use std::collections::HashMap;
use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};
use std::fmt;

use crate::bit_cask_key::BitCaskKey;
use crate::config::Config;
use crate::entry::{Entry, MappedStoredEntry};
use crate::errors::Error;
use crate::key_directory::{ KeyDirectory, Entry as KeyDirectoryEntry };
use crate::merge_config::MergeConfig;
use crate::merged_state::MergedState;
use crate::segments::Segments;
use crate::store::Store;


pub struct KVStore<Key: BitCaskKey, S: Store> {
    segments: Segments<S>,
    key_directory: KeyDirectory<Key>,
    lock: RwLock<()>,
    merge_config: MergeConfig<Key>,
    counter: u64,
}


impl<Key: BitCaskKey, S: Store > KVStore<Key, S> {
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

    pub fn update(&mut self, key: Key, value: Vec<u8>) -> Result<(), Error> {
        self.put(key, value)
    }

    pub fn delete(& mut self, key: Key) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        self.segments.append_deleted(key.clone())?;
        self.key_directory.delete(&key);
        Ok(())
    }


    pub fn get(&self, key: Key) -> Result<Vec<u8>, Error> {
        let _read_lock = self.lock.read().unwrap();
        if let Some(entry) = self.key_directory.get(&key) {
            let stored_entry = self.segments.read(entry.file_id, entry.offset, entry.entry_length)?;
            return Ok(stored_entry.value);
        }
        Err(Error::EntryNotFound)
    }

    // WriteBack writes back the changes (merged changes) to new inactive segments. This operation is performed during merge.
    // It writes all the changes into M new inactive segments and once those changes are written to the new inactive segment(s), the state of the keys present in the `changes` parameter is updated in the KeyDirectory. More on this is mentioned in Worker.go inside merge/ package.
    // Once the state is updated in the KeyDirectory, the old segments identified by `fileIds` are removed from disk.
    fn write_back(&mut self, file_ids: Vec<u64>, changes: HashMap<Key, MappedStoredEntry<Key>>) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        let write_back_responses = self.segments.write_back(changes)?;
        self.key_directory.bulk_update(write_back_responses);
        self.segments.remove(file_ids.as_slice());
        Ok(())
    }

    pub fn clear_log(&mut self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.remove_active();
        self.segments.remove_all_inactive();
    }

    fn sync(&self) {
        let _write_lock = self.lock.write().unwrap();
        self.segments.sync();
    }

    // pub fn shutdown(&mut self) {
    //     let _write_lock = self.lock.write().unwrap();
    //     self.segments.shutdown();
    // }

    fn reload(&mut self) -> Result<(), Error> {
        let _write_lock = self.lock.write().unwrap();
        for (file_id, segment) in self.segments.all_inactive_segments() {
            println!("this is file ID {:?}", file_id);
            let entries = segment.read_full(self.merge_config.key_mapper())?;
            self.key_directory.reload(*file_id, entries);
        }
        Ok(())
    }

    fn begin_merge(&mut self) -> Result<(), Error> {
        let (file_ids, segments)  = 
            if self.merge_config.should_read_all_segments() {
                self.segments.read_all_inactive_segments(self.merge_config.key_mapper())?
            } else {
                self.segments.read_inactive_segments(self.merge_config.total_segments_to_read(), self.merge_config.key_mapper())?
            };
            println!("old file ids {:?}", file_ids.clone());
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
