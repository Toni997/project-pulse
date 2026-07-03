use std::{cmp, sync::Arc};

use log::info;

use crate::{
    audio::{
        asset_pool::ASSET_POOL,
        snapshot::{clip_event::ClipEvent, project_snapshot::ProjectSnapshot},
    },
    core::types::{EngineSampleFormat, Id},
};

pub struct ClipNode {
    pub source_id: Id,
    pub source_offset_samples: usize,
}

impl ClipNode {
    pub fn render(
        &self,
        position_samples: usize,
        out: &mut [EngineSampleFormat],
        clip_event_node: &ClipEvent,
        snapshot: Arc<ProjectSnapshot>,
    ) -> usize {
        // TODO get track node and do render on it
        let pcm_data = ASSET_POOL.audio.get_pcm_by_id(&self.source_id);
        if pcm_data.is_none() {
            info!(
                "ClipNode: render: pcm_data is None for source_id: {}",
                self.source_id
            );
            return 0;
        }
        let pcm_data_unwrap = pcm_data.unwrap();
        let source_samples = pcm_data_unwrap.as_ref().samples();
        let needed_samples_count: usize = out.len();
        let mut dest_start_index = 0;
        if position_samples < clip_event_node.start_sample {
            dest_start_index = clip_event_node.start_sample - position_samples;
            out[0..dest_start_index].fill(0.0);
        }
        let source_start_index = position_samples.saturating_sub(clip_event_node.start_sample)
            + self.source_offset_samples;
        let source_end_index = cmp::min(
            source_start_index + needed_samples_count - dest_start_index,
            source_samples.len(),
        );
        let source_samples_slice = &source_samples[source_start_index..source_end_index];
        let dest_end_index = cmp::min(
            needed_samples_count,
            clip_event_node.end_sample.saturating_sub(position_samples),
        );
        out[dest_start_index..dest_end_index].copy_from_slice(source_samples_slice);
        source_samples_slice.len()
    }
}
