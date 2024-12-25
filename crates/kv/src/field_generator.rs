use std::sync::Arc;

use crate::{clock};

pub struct TimestampBasedFileIdGenerator {
    pub clock: Arc<dyn clock::Clock>,
}

impl TimestampBasedFileIdGenerator {
    pub fn next(&self) -> u64 {
        self.clock.monotonic_now()
    }
}