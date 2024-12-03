use wasi::clocks::{self, wall_clock};

#[derive(Copy, Clone)]
pub struct Duration(pub u64);

pub trait Clock {
    fn now(&self) ->  u64;
}

pub struct WasiClock;

impl Clock for WasiClock {
    fn now(&self) -> u64 {
        clocks::wall_clock::now().seconds
    }
}
