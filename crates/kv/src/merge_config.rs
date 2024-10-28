use crate::{bit_cask_key::BitCaskKey, clock::Duration};



pub struct MergeConfig<Key: BitCaskKey> {
    total_segments_to_read: usize,
    should_read_all_segments: bool,
    key_mapper: fn(&[u8]) -> Key,
    run_merge_every: Duration,
}

impl<Key: BitCaskKey> MergeConfig<Key> {
    pub fn new(total_segments_to_read: usize, key_mapper: fn(&[u8]) -> Key) -> Self {
        Self {
            total_segments_to_read,
            should_read_all_segments: false,
            key_mapper,
            run_merge_every: Duration(5 * 60), // 5 minutes in seconds
        }
    }

    pub fn new_with_duration(
        total_segments_to_read: usize,
        run_merge_every: Duration,
        key_mapper: fn(&[u8]) -> Key,
    ) -> Self {
        Self {
            total_segments_to_read,
            should_read_all_segments: false,
            key_mapper,
            run_merge_every,
        }
    }

    pub fn new_with_all_segments_to_read(key_mapper: fn(&[u8]) -> Key) -> Self {
        Self {
            total_segments_to_read: 0, // Not applicable in this case
            should_read_all_segments: true,
            key_mapper,
            run_merge_every: Duration(5 * 60), // 5 minutes
        }
    }

    pub fn new_with_all_segments_to_read_every_fixed_duration(
        run_merge_every: Duration,
        key_mapper: fn(&[u8]) -> Key,
    ) -> Self {
        Self {
            total_segments_to_read: 0, // Not applicable in this case
            should_read_all_segments: true,
            key_mapper,
            run_merge_every,
        }
    }

    pub fn total_segments_to_read(&self) -> usize {
        self.total_segments_to_read
    }

    pub fn should_read_all_segments(&self) -> bool {
        self.should_read_all_segments
    }

    pub fn key_mapper(&self) -> fn(&[u8]) -> Key {
        self.key_mapper
    }

    pub fn run_merge_every(&self) -> Duration {
        self.run_merge_every
    }
}