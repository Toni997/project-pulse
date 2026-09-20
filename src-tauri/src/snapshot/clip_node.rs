use std::sync::Arc;

use crate::{core::types::EngineSampleFormat, engine::asset_pool::AudioSamples};

pub struct ClipNode {
    samples: Arc<AudioSamples>,
    source_offset_samples: usize,
}

impl ClipNode {
    pub fn new(samples: Arc<AudioSamples>, source_offset_samples: usize) -> Self {
        Self {
            samples,
            source_offset_samples,
        }
    }

    /// Writes this clip's audio into `out`, the buffer that starts at
    /// `position_samples` on the timeline, for a clip occupying
    /// `start_sample..end_sample` (end exclusive). Only the part of the buffer
    /// the clip covers is touched. Clips on a track are assumed not to overlap;
    /// overlapping clips would overwrite each other.
    pub fn render(
        &self,
        position_samples: usize,
        out: &mut [EngineSampleFormat],
        start_sample: usize,
        end_sample: usize,
    ) {
        // TODO get track node and do render on it
        let buffer_end = position_samples + out.len();
        let from = position_samples.max(start_sample);
        let to = buffer_end.min(end_sample);
        if from >= to {
            return;
        }

        let source = self.samples.as_slice();
        let source_from = from - start_sample + self.source_offset_samples;
        if source_from >= source.len() {
            return;
        }
        let count = (to - from).min(source.len() - source_from);

        let dest_from = from - position_samples;
        out[dest_from..dest_from + count]
            .copy_from_slice(&source[source_from..source_from + count]);
    }
}
