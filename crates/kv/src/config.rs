use crate::{bit_cask_key::BitCaskKey, clock, merge_config::MergeConfig};


pub struct Config<K: BitCaskKey> {
    directory: String,
    max_segment_size_bytes: u64,
    key_directory_capacity: u64,
    merge_config: Option<Box<MergeConfig<K>>>,
    clock: Box<dyn clock::Clock>,
}

impl<K: BitCaskKey> Config<K> {
    pub fn new(
        directory: String,
        max_segment_size_bytes: u64,
        key_directory_capacity: u64,
        merge_config: Option<Box<MergeConfig<K>>>,
        clock: Box<dyn clock::Clock>
    ) -> Self {
        Self {
            directory,
            max_segment_size_bytes,
            key_directory_capacity,
            merge_config,
            clock,
        }
    }

    // Getter methods
    pub fn directory(&self) -> &str {
        &self.directory
    }

    pub fn max_segment_size_in_bytes(&self) -> u64 {
        self.max_segment_size_bytes
    }

    pub fn key_directory_capacity(&self) -> u64 {
        self.key_directory_capacity
    }

    pub fn clock(&self) -> &dyn clock::Clock {
        &*self.clock
    }

    pub fn merge_config(&self) -> Option<&MergeConfig<K>> {
        self.merge_config.as_deref()
    }
}