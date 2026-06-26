use std::sync::Arc;

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
            println!(
                "ClipNode: render: pcm_data is None for source_id: {}",
                self.source_id
            );
            return 0;
        }
        let pcm_data_unwrap = pcm_data.unwrap();
        let source_samples = pcm_data_unwrap.as_ref().samples();
        let needed_samples_count: usize = out.len();
        println!("ClipNode: render: position_samples: {}, needed_samples_count: {}, source_samples.len(): {}, clip_event_node.start_sample: {}, clip_event_node.end_sample: {}, self.source_offset_samples: {}",
            position_samples, needed_samples_count, source_samples.len(), clip_event_node.start_sample, clip_event_node.end_sample, self.source_offset_samples);
        // TODO fix the logic here to handle edge cases where the clip only has to fill the buffer partly
        let start_index =
            position_samples - clip_event_node.start_sample + self.source_offset_samples;
        let end_index = start_index + needed_samples_count;
        let source_samples_slice = &source_samples[start_index..end_index];
        out.copy_from_slice(source_samples_slice);
        source_samples_slice.len()
    }
}
