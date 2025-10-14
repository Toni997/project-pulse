pub struct TimelineEvent {
    track_id: usize,
    offset: usize,
}

impl TimelineEvent {
    pub fn new(track_id: usize, offset: usize) -> Self {
        Self { track_id, offset }
    }

    pub fn apply(&self) {
        // Logic to apply the event at the specified offset
    }
}
