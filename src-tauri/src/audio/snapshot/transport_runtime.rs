pub struct TransportRuntime {
    pub is_playing: bool,
    pub loop_range_ppq: Option<(usize, usize)>,
}

impl TransportRuntime {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            loop_range_ppq: None,
        }
    }
}
