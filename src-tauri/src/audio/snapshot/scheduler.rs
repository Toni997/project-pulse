use crate::audio::snapshot::clip_event::ClipEvent;
use crate::audio::{asset_pool::ASSET_POOL, engine::AUDIO_ENGINE, project_state::PROJECT_STATE};
use sorted_vec::SortedVec;
use tauri::async_runtime;

// #[derive(Debug, Eq)]
// pub enum SchedulerEvent {
//     Clip(ClipEvent),
//     Midi(MidiEvent),
//     Automation(AutomationEvent),
// }

pub struct Scheduler {
    pub events: SortedVec<ClipEvent>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            events: SortedVec::new(),
        }
    }

    pub async fn build() -> Self {
        let new_scheduler = async_runtime::spawn_blocking(move || -> Scheduler {
            let mut new_scheduler = Scheduler::new();

            let tempo_bpm = PROJECT_STATE.tempo_bpm() as f64;
            let ppq = PROJECT_STATE.ppq() as f64;
            let sample_rate = AUDIO_ENGINE.sample_rate() as f64;
            let channels = AUDIO_ENGINE.num_channels();

            PROJECT_STATE.with_tracks(|tracks| {
                for track in tracks.values() {
                    let clips = match track {
                        crate::audio::track::GeneratorTrack::AudioTrack(t) => &t.clips,
                        crate::audio::track::GeneratorTrack::SamplerTrack(t) => &t.clips,
                    };

                    for clip in clips.values() {
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

                        new_scheduler.events.push(ClipEvent {
                            start_sample,
                            end_sample,
                            node_id: clip.id.clone(),
                        });
                    }
                }
            });

            new_scheduler
        })
        .await
        .unwrap();

        new_scheduler
    }
}
