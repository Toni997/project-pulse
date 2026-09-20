use std::sync::Arc;

use anyhow::Error;
use log::info;
use tauri::async_runtime;

use crate::core::notify::log_and_notify_error;
use crate::core::types::Id;
use crate::engine::asset_pool::AssetPool;
use crate::engine::decoder::decode_audio_file;
use crate::engine::device::AudioEngine;
use crate::project::clip::{Clip, ClipToInsert, NewClip};
use crate::project::project_state::ProjectState;
use crate::project::track::{AudioTrack, SamplerTrack};

/// Application-level editing operations that need more than the project
/// model: loading/decoding audio into the asset pool, resolving clip lengths
/// against the audio device, and reporting failures to the user. Once
/// everything is resolved it hands finished values to `ProjectState`, which
/// stays plain synchronous data.
pub struct ProjectEditor {
    project_state: Arc<ProjectState>,
    asset_pool: Arc<AssetPool>,
    audio_engine: Arc<AudioEngine>,
}

impl ProjectEditor {
    pub fn new(
        project_state: Arc<ProjectState>,
        asset_pool: Arc<AssetPool>,
        audio_engine: Arc<AudioEngine>,
    ) -> Self {
        Self {
            project_state,
            asset_pool,
            audio_engine,
        }
    }

    pub async fn add_sampler_track(&self, source_path: Option<String>) -> Option<SamplerTrack> {
        let source_id = match source_path {
            Some(path) => Some(self.load_audio(path).await?.id),
            None => None,
        };
        Some(self.project_state.add_sampler_track(source_id))
    }

    pub async fn add_audio_track_with_clip(&self, clip: ClipToInsert) -> Option<AudioTrack> {
        info!("ProjectEditor: add_audio_track_with_clip: {:?}", clip);
        let audio = self.load_audio(clip.source_path).await?;
        let new_clip = self.new_clip(audio, clip.start_ppq);
        Some(self.project_state.add_audio_track_with_clip(new_clip))
    }

    pub async fn add_clip_to_audio_track(&self, clip: ClipToInsert) -> Option<Clip> {
        info!("ProjectEditor: add_clip_to_audio_track: {:?}", clip);
        let Some(track_id) = clip.track_id else {
            log_and_notify_error("Error trying to insert clip: missing trackId");
            return None;
        };
        let audio = self.load_audio(clip.source_path).await?;
        let new_clip = self.new_clip(audio, clip.start_ppq);
        match self
            .project_state
            .add_clip_to_audio_track(&track_id, new_clip)
        {
            Ok(clip) => Some(clip),
            Err(e) => {
                log_and_notify_error(&format!("Error trying to insert clip: {e}"));
                None
            }
        }
    }

    pub async fn assign_source_to_sampler_track(
        &self,
        track_id: Id,
        source_path: Option<String>,
    ) -> Option<Id> {
        let audio = self.load_audio(source_path?).await?;
        match self
            .project_state
            .assign_source_to_sampler_track(&track_id, audio.id.clone())
        {
            Ok(()) => Some(audio.id),
            Err(e) => {
                log_and_notify_error(&format!("Error trying to assign audio: {e}"));
                None
            }
        }
    }

    /// Returns the already-loaded asset for `source_path`, or decodes it.
    /// Reports failures to the user and returns `None`.
    async fn load_audio(&self, source_path: String) -> Option<LoadedAudio> {
        if let Some((id, asset)) = self.asset_pool.audio.get_by_path(&source_path) {
            return Some(LoadedAudio {
                id,
                num_samples: asset.num_samples(),
                name: asset.display_name(),
            });
        }

        // Decoding can be CPU/IO heavy, so it runs on a blocking thread.
        let audio_engine = Arc::clone(&self.audio_engine);
        let decode_result =
            async_runtime::spawn_blocking(move || decode_audio_file(source_path, &audio_engine))
                .await
                .unwrap_or_else(|e| Err(Error::new(e)));

        match decode_result {
            Ok(decoded) => {
                let num_samples = decoded.data.len();
                let name = decoded.file_name.clone();
                let id = self.asset_pool.audio.add(decoded);
                Some(LoadedAudio {
                    id,
                    num_samples,
                    name,
                })
            }
            Err(e) => {
                log_and_notify_error(&format!("Error trying to load audio file: {e}"));
                None
            }
        }
    }

    fn clip_length_ppq(&self, num_samples: usize) -> usize {
        let frames = (num_samples / self.audio_engine.num_channels()) as f64;
        let sample_rate = self.audio_engine.sample_rate() as f64;
        let bpm = self.project_state.tempo_bpm() as f64;
        let ppq = self.project_state.ppq() as f64;
        ((frames * bpm * ppq) / (sample_rate * 60.0)).round() as usize
    }

    fn new_clip(&self, audio: LoadedAudio, start_ppq: usize) -> NewClip {
        NewClip {
            length_ppq: self.clip_length_ppq(audio.num_samples),
            name: audio.name,
            start_ppq,
            source_id: audio.id,
        }
    }
}

struct LoadedAudio {
    id: Id,
    num_samples: usize,
    name: String,
}
