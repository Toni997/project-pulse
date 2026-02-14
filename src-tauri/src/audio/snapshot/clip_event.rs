use std::cmp::Ordering;

#[derive(Debug, Eq)]
pub struct ClipEvent {
    pub start_sample: usize,
    pub end_sample: usize,
    pub source_node_id: String,
}

impl Ord for ClipEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_sample
            .cmp(&other.start_sample)
            .then_with(|| self.end_sample.cmp(&other.end_sample))
            .then_with(|| self.source_node_id.cmp(&other.source_node_id))
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
            && self.source_node_id == other.source_node_id
    }
}
