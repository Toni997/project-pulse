use std::sync::atomic::{AtomicBool, AtomicUsize};

pub struct Transport {
    pub is_playing: AtomicBool,
    pub position_ppq: AtomicUsize,
    pub loop_range_ppq: (AtomicUsize, AtomicUsize),
}

impl Transport {
    pub fn default() -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            position_ppq: AtomicUsize::new(0),
            loop_range_ppq: (AtomicUsize::new(0), AtomicUsize::new(0)),
        }
    }
}
