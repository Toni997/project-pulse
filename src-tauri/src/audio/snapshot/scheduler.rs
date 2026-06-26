use crate::audio::snapshot::clip_event::ClipEvent;
use crate::audio::snapshot::project_snapshot::ProjectSnapshot;
use crate::audio::track::GeneratorTrack;
use crate::audio::{asset_pool::ASSET_POOL, engine::AUDIO_ENGINE, project_state::PROJECT_STATE};
use crate::core::types::Id;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

// #[derive(Debug, Eq)]
// pub enum SchedulerEvent {
//     Clip(ClipEvent),
//     Midi(MidiEvent),
//     Automation(AutomationEvent),
// }

pub struct SchedulerAudioTrack {
    pub id: Id,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub clips: Vec<ClipEvent>,
}

impl SchedulerAudioTrack {
    pub fn fill_with_active_samples(
        &self,
        position_samples: usize,
        last_active_clip_index: &mut usize,
        out: &mut Vec<f32>,
        snapshot: Arc<ProjectSnapshot>,
    ) {
        out.fill(0.0);
        println!(
            "SchedulerAudioTrack: {} filling with active samples, {}",
            self.name,
            self.clips.len()
        );
        if self.clips.is_empty() {
            return;
        }
        let mut filled_samples_count = 0;
        let needed_samples_count = out.len();
        let mut current_clip = self.clips.get(*last_active_clip_index);
        if current_clip.is_none()
            || !current_clip
                .unwrap()
                .should_be_rendered(position_samples, needed_samples_count)
        {
            *last_active_clip_index = 0;
            current_clip = self.clips.get(*last_active_clip_index);
        }

        while filled_samples_count < needed_samples_count
            && current_clip.is_some()
            && current_clip
                .unwrap()
                .should_be_rendered(position_samples, needed_samples_count)
        {
            let clip = current_clip.unwrap();
            filled_samples_count = clip.render(
                position_samples,
                &mut out[filled_samples_count..needed_samples_count],
                snapshot.clone(),
            );
            *last_active_clip_index += 1;
            current_clip = self.clips.get(*last_active_clip_index);
        }
    }
}

pub struct Scheduler {
    pub tracks: Vec<Arc<SchedulerAudioTrack>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn build(should_abort: impl Fn() -> bool) -> Option<Self> {
        println!("Scheduler: build called");
        if should_abort() {
            return None;
        }

        let mut new_scheduler = Scheduler::new();
        let aborted = AtomicBool::new(false);

        let tempo_bpm = PROJECT_STATE.tempo_bpm() as f64;
        let ppq = PROJECT_STATE.ppq() as f64;
        let sample_rate = AUDIO_ENGINE.sample_rate() as f64;
        let channels = AUDIO_ENGINE.num_channels();

        PROJECT_STATE.with_tracks(|tracks| {
            for track in tracks.values() {
                if should_abort() {
                    aborted.store(true, Ordering::SeqCst);
                    return;
                }

                let (id, name, volume, pan, muted, clips) = match track {
                    GeneratorTrack::AudioTrack(t) => {
                        (&t.id, &t.name, t.volume, t.pan, t.muted, &t.clips)
                    }
                    GeneratorTrack::SamplerTrack(t) => {
                        (&t.id, &t.name, t.volume, t.pan, t.muted, &t.clips)
                    }
                };

                let mut scheduler_track = SchedulerAudioTrack {
                    id: id.clone(),
                    name: name.clone(),
                    volume,
                    pan,
                    muted,
                    clips: Vec::new(),
                };

                for clip in clips.values() {
                    if should_abort() {
                        aborted.store(true, Ordering::SeqCst);
                        return;
                    }

                    let total_samples = ASSET_POOL
                        .audio
                        .get_num_samples_by_id(&clip.source_id)
                        .unwrap_or(0);
                    if total_samples == 0 {
                        continue;
                    }

                    let total_frames = total_samples / channels;
                    let total_samples_aligned = total_frames * channels;

                    let offset_frames = clip.source_offset_samples / channels;
                    let offset_samples_aligned = offset_frames * channels;

                    let remaining_samples =
                        total_samples_aligned.saturating_sub(offset_samples_aligned);
                    if remaining_samples == 0 {
                        continue;
                    }

                    let beats = clip.start_ppq as f64 / ppq;
                    let seconds = (beats * 60.0) / tempo_bpm;
                    let start_frames = (seconds * sample_rate).round() as usize;
                    let start_sample = start_frames * channels;
                    let end_sample = start_sample.saturating_add(remaining_samples);

                    scheduler_track.clips.push(ClipEvent {
                        start_sample,
                        end_sample,
                        node_id: clip.id.clone(),
                    });
                }

                scheduler_track.clips.sort();
                new_scheduler.tracks.push(Arc::new(scheduler_track));
            }
        });

        if aborted.load(Ordering::SeqCst) {
            return None;
        }

        Some(new_scheduler)
    }
}
