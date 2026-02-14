use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    LazyLock,
};

pub struct Transport {
    pub is_playing: AtomicBool,
    pub position_ppq: AtomicUsize,
    pub loop_range_ppq: (AtomicUsize, AtomicUsize),
}

impl Transport {
    pub fn new() -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            position_ppq: AtomicUsize::new(0),
            loop_range_ppq: (AtomicUsize::new(0), AtomicUsize::new(0)),
        }
    }

    pub fn play(&self) {
        TRANSPORT.is_playing.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

pub static TRANSPORT: LazyLock<Transport> = LazyLock::new(|| Transport::new());
