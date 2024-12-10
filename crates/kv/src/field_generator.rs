use std::sync::Arc;

use crate::{bit_cask_key::BitCaskKey, clock, merge_config::MergeConfig};

pub struct TimestampBasedFileIdGenerator {
    pub clock: Arc<dyn clock::Clock>,
}

impl TimestampBasedFileIdGenerator {
    pub fn next(&self) -> u64 {
        self.clock.monotonic_now()
    }
}