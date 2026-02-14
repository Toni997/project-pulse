use sorted_vec::SortedVec;

use crate::audio::snapshot::clip_event::ClipEvent;

pub struct Scheduler {
    pub events: SortedVec<ClipEvent>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            events: SortedVec::new(),
        }
    }
}
