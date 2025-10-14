use crate::audio::timeline_event::TimelineEvent;

use std::collections::BTreeMap;

pub struct Arrangement {
    title: String,
    timeline: BTreeMap<usize, TimelineEvent>,
}

impl Arrangement {
    pub fn new(title: String) -> Self {
        Self {
            title,
            timeline: BTreeMap::new(),
        }
    }
}
