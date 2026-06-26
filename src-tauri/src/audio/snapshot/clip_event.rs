use std::{cmp::Ordering, sync::Arc};

use crate::{
    audio::snapshot::project_snapshot::ProjectSnapshot,
    core::types::{EngineSampleFormat, Id},
};

#[derive(Debug, Eq)]
pub struct ClipEvent {
    pub start_sample: usize,
    pub end_sample: usize,
    pub node_id: Id,
}

impl Ord for ClipEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_sample
            .cmp(&other.start_sample)
            .then_with(|| self.end_sample.cmp(&other.end_sample))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl PartialOrd for ClipEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ClipEvent {
    fn eq(&self, other: &Self) -> bool {
        self.start_sample == other.start_sample
            && self.end_sample == other.end_sample
            && self.node_id == other.node_id
    }
}

impl ClipEvent {
    pub fn should_be_rendered(&self, position_samples: usize, needed_samples_count: usize) -> bool {
        position_samples + needed_samples_count > self.start_sample
            && position_samples < self.end_sample
    }

    pub fn render(
        &self,
        position_samples: usize,
        out: &mut [EngineSampleFormat],
        snapshot: Arc<ProjectSnapshot>,
    ) -> usize {
        let data_nodes = snapshot.get_data_nodes();
        let node = data_nodes
            .nodes
            .get(&self.node_id)
            .and_then(|node| node.as_clip());
        if node.is_none() {
            return 0;
        }
        let clip_node = node.unwrap();
        let filled_samples_count = clip_node.render(position_samples, out, &self, snapshot.clone());
        filled_samples_count
    }
}
