use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use crate::clock::Clock;
use crate::bit_cask_key::BitCaskKey;
use crate::entry::{Entry, MappedStoredEntry, StoredEntry};
use crate::errors::Error;
use crate::field_generator::TimestampBasedFileIdGenerator;
use crate::segment::{segment_name, AppendEntryResponse, Segment, SEGMENT_FILE_PREFIX, SEGMENT_FILE_SUFFIX};
use crate::store::Store;


pub struct Segments<S: Store> {
    active_segment: Segment<S>,
    inactive_segments: HashMap<u64, Segment<S>>,
    file_id_generator: TimestampBasedFileIdGenerator,
    clock: Arc<dyn Clock>,
    max_segment_size_bytes: u64,
    directory: String,
}

pub struct WriteBackResponse<K> {
    pub key: K,
    pub append_entry_response: AppendEntryResponse,
}

impl<S: Store > Segments<S> {
    pub fn new<K: BitCaskKey>(directory: String, max_segment_size_bytes: u64, clock: Arc<dyn Clock>) -> Result<Self, Error> {
        let file_id_generator = TimestampBasedFileIdGenerator{ clock: clock.clone()};
        let file_id = file_id_generator.next();
        let active_segment = Segment::new(file_id, &directory)?;

        let mut segments = Segments {
            active_segment,
            inactive_segments: HashMap::new(),
            file_id_generator,
            clock,
            max_segment_size_bytes,
            directory,
        };
        segments.reload::<K>()?;
        Ok(segments)
    }

    pub fn  append<K: BitCaskKey>(&mut self, key: K, value: Vec<u8>) -> Result<AppendEntryResponse, Error> {
        self.maybe_rollover_active_segment()?;
        self.active_segment.append(&Entry::new(key, value, self.clock.clone()))
    }

    pub fn append_deleted<K: BitCaskKey>(&mut self, key: K) -> Result<AppendEntryResponse, Error> {
        self.maybe_rollover_active_segment()?;
        self.active_segment.append(&Entry::new_deleted_entry(key, self.clock.clone()))
    }

    pub fn read(&self, file_id: u64, offset: i64, size: u32) -> Result<StoredEntry, Error> {
        if file_id == self.active_segment.file_id {
            return self.active_segment.read(offset, size);
        }
        if let Some(segment) = self.inactive_segments.get(&file_id) {
            segment.read(offset, size)
        } else {
            Err(Error::FileNotFound(file_id))
        }
    }

    pub fn read_inactive_segments<K: BitCaskKey>(
        &self,
        total_segments: usize,
        key_mapper: fn(&[u8]) -> K,
    ) -> Result<(Vec<u64>, Vec<Vec<MappedStoredEntry<K>>>), Error> {
        let mut index = 0;
        let mut contents = Vec::with_capacity(total_segments);
        let mut file_ids = Vec::with_capacity(total_segments);

        for (file_id, segment) in &self.inactive_segments {
            if index >= total_segments {
                break;
            }
            let entries = segment.read_full(key_mapper)?;
            contents.push(entries);
            file_ids.push(*file_id);
            index += 1;
        }
        Ok((file_ids, contents))
    }

    pub fn read_all_inactive_segments<K: BitCaskKey>(
        &self,
        key_mapper: fn(&[u8]) -> K,
    ) -> Result<(Vec<u64>, Vec<Vec<MappedStoredEntry<K>>>), Error> {
        self.read_inactive_segments(self.inactive_segments.len(), key_mapper)
    }

    pub fn write_back<K: BitCaskKey + Clone>(
        &mut self,
        changes: HashMap<K, MappedStoredEntry<K>>,
    ) -> Result<Vec<WriteBackResponse<K>>, Error> {
        let mut segment = Segment::<S>::new(self.file_id_generator.next(), &self.directory)?;
        self.inactive_segments.insert(segment.file_id, segment.clone());

        let mut write_back_responses = Vec::with_capacity(changes.len());
        for (key, value) in changes {
            let append_entry_response = segment.append(&Entry::new_preserving_timestamp(
                key.clone(),
                value.value,
                value.timestamp,
                self.clock.clone(),
            ))?;
            write_back_responses.push(WriteBackResponse {
                key,
                append_entry_response,
            });

            if let Some(new_segment) = self.maybe_rollover_segment(&mut segment)? {
                self.inactive_segments.insert(new_segment.file_id, new_segment.clone());
                segment = new_segment;
            }
        }
        Ok(write_back_responses)
    }

    pub fn remove_active(&mut self) {
        self.active_segment.remove();
    }

    pub fn remove_all_inactive(&mut self) {
        for segment in self.inactive_segments.values_mut() {
            segment.remove();
        }
        self.inactive_segments.clear();
    }

    pub fn remove(&mut self, file_ids: &[u64]) {
        for file_id in file_ids {
            if let Some(mut segment) = self.inactive_segments.remove(file_id) {
                segment.remove();
            }
        }
    }

    pub fn all_inactive_segments(&self) -> &HashMap<u64, Segment<S>> {
        &self.inactive_segments
    }

    pub fn sync(&self) {
        self.active_segment.sync();
        for segment in self.inactive_segments.values() {
            segment.sync();
        }
    }

    // pub fn shutdown(&mut self) {
    //     self.active_segment = Segment::default(); // Implement Default for Segment if necessary
    //     self.inactive_segments.clear();
    // }

    fn maybe_rollover_active_segment(&mut self) -> Result<(), Error> {
        if let Some(new_segment) = self.maybe_rollover_segment(&mut self.active_segment.clone())? {
            self.inactive_segments.insert(self.active_segment.file_id, self.active_segment.clone());
            self.active_segment = new_segment;
        }
        Ok(())
    }

    fn maybe_rollover_segment(&self, segment: &mut Segment<S>) -> Result<Option<Segment<S>>, Error> {
        if segment.size_in_bytes() >= self.max_segment_size_bytes as i64 {
            // TODO: Fix
            // segment.stop_writes();
            let new_segment = Segment::new(self.file_id_generator.next(), &self.directory)?;
            Ok(Some(new_segment))
        } else {
            Ok(None)
        }
    }

    fn reload<K: BitCaskKey>(&mut self) -> Result<(), Error> {
        let suffix = format!("{}.{}", SEGMENT_FILE_PREFIX, SEGMENT_FILE_SUFFIX);
 
        for entry in S::get_files(&self.directory)? {
            if entry.ends_with(&suffix) {
                let file_id_str = entry.split('_').next().ok_or(Error::ParseError)?;
                let file_id: u64 = file_id_str.parse().map_err(|_| Error::ParseError)?;

                if file_id != self.active_segment.file_id {
                    let segment = Self::reload_inactive_segment::<K>(file_id, &self.directory)?;
                    self.inactive_segments.insert(file_id, segment);
                }
            }
        }
            
        Ok(())
    }

    fn reload_inactive_segment<K: BitCaskKey>(file_id: u64, directory: &str) -> Result<Segment<S>, Error> {
        let file_path = segment_name(file_id);
        let store = S::open(&file_path, directory)?;
        
        Ok(Segment {
            file_id,
            file_path,
            store,
        })
    }
}