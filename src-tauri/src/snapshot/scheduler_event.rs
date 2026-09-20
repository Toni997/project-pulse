use std::cmp::Ordering;

use crate::{
    core::types::{EngineSampleFormat, Id},
    snapshot::{clip_node::ClipNode, data_nodes::DataNodes},
};

/// Something the scheduler plays over a span of samples. The event holds only
/// its timing and which node to play; the node's data lives in `DataNodes`, so
/// changing that data never requires rebuilding the schedule.
pub struct SchedulerEvent {
    node_id: Id,
    node_kind: SchedulerNodeKind,
    start_sample: usize,
    end_sample: usize,
}

/// Which kind of node an event's `node_id` refers to, and so which lookup to
/// use on `DataNodes`.
#[derive(Clone, Copy)]
pub enum SchedulerNodeKind {
    Clip,
}

impl Ord for SchedulerEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_sample
            .cmp(&other.start_sample)
            .then_with(|| self.end_sample.cmp(&other.end_sample))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl PartialOrd for SchedulerEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SchedulerEvent {
    fn eq(&self, other: &Self) -> bool {
        self.start_sample == other.start_sample
            && self.end_sample == other.end_sample
            && self.node_id == other.node_id
    }
}

impl Eq for SchedulerEvent {}

impl SchedulerEvent {
    pub fn new(
        node_id: Id,
        node_kind: SchedulerNodeKind,
        start_sample: usize,
        end_sample: usize,
    ) -> Self {
        Self {
            node_id,
            node_kind,
            start_sample,
            end_sample,
        }
    }

    /// Whether this event shares at least one sample with the buffer that
    /// starts at `position_samples` and is `needed_samples_count` long. The
    /// event's end is exclusive, so touching without overlapping doesn't count.
    pub fn is_scheduled_at_position(
        &self,
        position_samples: usize,
        needed_samples_count: usize,
    ) -> bool {
        position_samples < self.end_sample
            && self.start_sample < position_samples + needed_samples_count
    }

    /// Whether this event starts at or after the end of the buffer, so it
    /// can't affect that buffer or any earlier one.
    pub fn is_scheduled_after_buffer(
        &self,
        position_samples: usize,
        needed_samples_count: usize,
    ) -> bool {
        self.start_sample >= position_samples + needed_samples_count
    }

    /// Writes this event's audio into `out`, the buffer starting at
    /// `position_samples`. Renders nothing if the node it refers to is missing.
    pub fn render(
        &self,
        position_samples: usize,
        out: &mut [EngineSampleFormat],
        data_nodes: &DataNodes,
    ) {
        match self.node_kind {
            SchedulerNodeKind::Clip => {
                if let Some(clip_node) = data_nodes.get_node::<ClipNode>(&self.node_id) {
                    clip_node.render(position_samples, out, self.start_sample, self.end_sample);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/snapshot/scheduler_event.rs"]
mod tests;
