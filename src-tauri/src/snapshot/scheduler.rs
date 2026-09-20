use crate::core::types::Id;
use crate::engine::asset_pool::AssetPool;
use crate::engine::device::AudioEngine;
use crate::project::project_state::ProjectState;
use crate::project::track::TrackCommon;
use crate::snapshot::data_nodes::DataNodes;
use crate::snapshot::scheduler_event::{SchedulerEvent, SchedulerNodeKind};
use std::sync::Arc;

pub struct SchedulerAudioTrack {
    id: Id,
    name: String,
    volume: f32,
    pan: f32,
    muted: bool,
    clips: Vec<SchedulerEvent>,
}

impl SchedulerAudioTrack {
    /// Renders every clip that overlaps the buffer starting at
    /// `position_samples` into `out`, which is cleared first.
    pub fn render(&self, position_samples: usize, out: &mut [f32], data_nodes: &DataNodes) {
        out.fill(0.0);
        let needed_samples_count = out.len();

        // TODO skip past clips faster than scanning from the start
        for clip in &self.clips {
            // Clips are sorted by start, so nothing after this one can play yet.
            if clip.is_scheduled_after_buffer(position_samples, needed_samples_count) {
                break;
            }
            if clip.is_scheduled_at_position(position_samples, needed_samples_count) {
                clip.render(position_samples, out, data_nodes);
            }
        }
    }
}

pub struct Scheduler {
    tracks: Vec<Arc<SchedulerAudioTrack>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn tracks(&self) -> &[Arc<SchedulerAudioTrack>] {
        &self.tracks
    }

    /// Builds the schedule one track at a time, holding the project's lock only
    /// while reading a single track, and gives up (`None`) as soon as
    /// `should_abort` says the project has changed underneath it.
    pub fn build(
        project_state: &ProjectState,
        asset_pool: &AssetPool,
        audio_engine: &AudioEngine,
        should_abort: impl Fn() -> bool,
    ) -> Option<Self> {
        if should_abort() {
            return None;
        }

        let timing = Timing {
            tempo_bpm: project_state.tempo_bpm() as f64,
            ppq: project_state.ppq() as f64,
            sample_rate: audio_engine.sample_rate() as f64,
            channels: audio_engine.num_channels(),
        };

        let mut new_scheduler = Scheduler::new();
        for track_id in project_state.track_ids() {
            if should_abort() {
                return None;
            }

            // `None` from `with_track` means the track was deleted since its id
            // was listed, i.e. an edit happened; `None` from inside means abort.
            let scheduler_track = project_state
                .with_track(&track_id, |track| {
                    Self::build_track(track.common(), asset_pool, &timing, &should_abort)
                })
                .flatten()?;
            new_scheduler.tracks.push(Arc::new(scheduler_track));
        }

        Some(new_scheduler)
    }

    /// `None` if the build was aborted part way through.
    fn build_track(
        track: &TrackCommon,
        asset_pool: &AssetPool,
        timing: &Timing,
        should_abort: &impl Fn() -> bool,
    ) -> Option<SchedulerAudioTrack> {
        let Timing {
            tempo_bpm,
            ppq,
            sample_rate,
            channels,
        } = *timing;

        let mut scheduler_track = SchedulerAudioTrack {
            id: track.id.clone(),
            name: track.name.clone(),
            volume: track.volume,
            pan: track.pan,
            muted: track.muted,
            clips: Vec::new(),
        };

        for clip in track.clips.values() {
            if should_abort() {
                return None;
            }

            let total_samples = asset_pool
                .audio
                .get_samples_by_id(&clip.source_id)
                .map_or(0, |samples| samples.len());
            if total_samples == 0 {
                continue;
            }

            let total_frames = total_samples / channels;
            let total_samples_aligned = total_frames * channels;

            let offset_frames = clip.source_offset_samples / channels;
            let offset_samples_aligned = offset_frames * channels;

            let remaining_samples = total_samples_aligned.saturating_sub(offset_samples_aligned);
            if remaining_samples == 0 {
                continue;
            }

            let beats = clip.start_ppq as f64 / ppq;
            let seconds = (beats * 60.0) / tempo_bpm;
            let start_frames = (seconds * sample_rate).round() as usize;
            let start_sample = start_frames * channels;
            let end_sample = start_sample.saturating_add(remaining_samples);

            scheduler_track.clips.push(SchedulerEvent::new(
                clip.id.clone(),
                SchedulerNodeKind::Clip,
                start_sample,
                end_sample,
            ));
        }

        scheduler_track.clips.sort();
        Some(scheduler_track)
    }
}

/// What turns a clip's position on the timeline into a sample position.
#[derive(Clone, Copy)]
struct Timing {
    tempo_bpm: f64,
    ppq: f64,
    sample_rate: f64,
    channels: usize,
}

#[cfg(test)]
#[path = "../tests/snapshot/scheduler.rs"]
mod tests;
