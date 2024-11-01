use crate::{bit_cask_key::BitCaskKey, clock, merge_config::MergeConfig};

pub struct TimestampBasedFileIdGenerator {
    clock: Box<dyn clock::Clock>,
}

impl TimestampBasedFileIdGenerator {
    fn next(&self) -> u64 {
        self.clock.now()
    }
}