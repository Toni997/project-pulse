use anyhow::Error;
use atomic_float::AtomicF32;
use indexmap::IndexMap;
use std::path::Path;
use std::sync::atomic::AtomicU16;
use std::sync::{atomic::Ordering, LazyLock, Mutex};
use tauri::async_runtime;

use crate::audio::asset_pool::ASSET_POOL;
use crate::audio::clip::{Clip, ClipToInsert};
use crate::audio::decoder::decode_audio_file;
use crate::audio::engine::AUDIO_ENGINE;
use crate::audio::preview_mixer::PREVIEW_MIXER;
use crate::audio::track::{
    default_track_name, AudioTrack, BusTrack, GeneratorTrack, MasterTrack, SamplerTrack,
};
use crate::core::constants::{
    PPQ_DEFAULT, TEMPO_BPM_DEFAULT, TIME_SIGNATURE_DENOMINATOR_DEFAULT,
    TIME_SIGNATURE_NUMERATOR_DEFAULT,
};
use crate::core::notify::log_and_notify_error;
use crate::core::types::Id;

pub struct ProjectState {
    ppq: AtomicU16,
    tempo_bpm: AtomicF32,
    time_signature: (u8, u8),
    master: Mutex<MasterTrack>,
    tracks: Mutex<IndexMap<Id, GeneratorTrack>>,
    buses: Mutex<IndexMap<Id, BusTrack>>,
}

impl ProjectState {
    fn display_name_from_path(source_path: &str) -> String {
        Path::new(source_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    async fn ensure_audio_asset(&self, source_path: String) -> Option<(Id, usize, String)> {
        if let Some(existing_id) = ASSET_POOL.audio.get_id_by_path(&source_path) {
            let num_samples = ASSET_POOL
                .audio
                .get_num_samples_by_id(&existing_id)
                .unwrap_or(0);
            let name = ASSET_POOL
                .audio
                .get_display_name_by_id(&existing_id)
                .unwrap_or_else(|| Self::display_name_from_path(&source_path));
            return Some((existing_id, num_samples, name));
        }

        let decode_result = async_runtime::spawn_blocking(move || decode_audio_file(source_path))
            .await
            .unwrap_or_else(|e| Err(Error::new(e)));

        match decode_result {
            Ok(decoded_audio_data) => {
                let num_samples = decoded_audio_data.data.len();
                let name = decoded_audio_data.file_name.clone();
                let asset_id = ASSET_POOL.audio.add(decoded_audio_data);
                Some((asset_id, num_samples, name))
            }
            Err(e) => {
                log_and_notify_error(format!("Error trying to load audio file: {e}"));
                None
            }
        }
    }

    fn calc_clip_length_ppq(&self, num_samples: usize) -> usize {
        let engine_channels = AUDIO_ENGINE.num_channels();
        let engine_sample_rate = AUDIO_ENGINE.sample_rate();

        let frames = (num_samples / engine_channels) as f64;
        let sr = engine_sample_rate as f64;
        let bpm = self.tempo_bpm.load(Ordering::SeqCst) as f64;
        let ppq = self.ppq.load(Ordering::SeqCst) as f64;
        let length_ppq = ((frames * bpm * ppq) / (sr * 60.0)).round() as usize;
        length_ppq
    }

    pub fn ppq(&self) -> u16 {
        self.ppq.load(Ordering::SeqCst)
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm.load(Ordering::SeqCst)
    }

    pub fn new() -> Self {
        Self {
            ppq: AtomicU16::new(PPQ_DEFAULT),
            tempo_bpm: AtomicF32::new(TEMPO_BPM_DEFAULT),
            time_signature: (
                TIME_SIGNATURE_NUMERATOR_DEFAULT,
                TIME_SIGNATURE_DENOMINATOR_DEFAULT,
            ),
            master: Mutex::new(MasterTrack::new()),
            tracks: Mutex::new(IndexMap::new()),
            buses: Mutex::new(IndexMap::new()),
        }
    }

    pub async fn add_sampler_track(&self, source_path: Option<String>) -> Option<SamplerTrack> {
        let asset_id = if let Some(source_path) = source_path {
            if let Some(existing_id) = ASSET_POOL.audio.get_id_by_path(&source_path) {
                Some(existing_id)
            } else {
                // Audio decoding can be CPU/IO heavy, so move it onto a blocking thread.
                let decode_result =
                    async_runtime::spawn_blocking(move || decode_audio_file(source_path))
                        .await
                        .unwrap_or_else(|e| Err(Error::new(e)));

                match decode_result {
                    Ok(decoded_audio_data) => Some(ASSET_POOL.audio.add(decoded_audio_data)),
                    Err(e) => {
                        log_and_notify_error(format!("Error trying to load audio file: {e}"));
                        return None;
                    }
                }
            }
        } else {
            None
        };

        let mut tracks = self.tracks.lock().unwrap();
        let track = SamplerTrack::new(default_track_name(tracks.len() + 1), asset_id);
        tracks.insert(
            track.id.clone(),
            GeneratorTrack::SamplerTrack(track.clone()),
        );
        Some(track)
    }

    pub fn add_audio_track(&self) -> AudioTrack {
        let mut tracks = self.tracks.lock().unwrap();
        let track = AudioTrack::new(default_track_name(tracks.len() + 1));
        tracks.insert(track.id.clone(), GeneratorTrack::AudioTrack(track.clone()));
        track
    }

    pub async fn add_audio_track_with_clip(&self, clip: ClipToInsert) -> Option<AudioTrack> {
        let (asset_id, num_samples, clip_name) = self.ensure_audio_asset(clip.source_path).await?;
        let length_ppq = self.calc_clip_length_ppq(num_samples);

        let mut tracks = self.tracks.lock().unwrap();
        let mut track = AudioTrack::new(default_track_name(tracks.len() + 1));

        let new_clip = Clip::new(
            track.id.clone(),
            clip_name,
            0,
            clip.start_ppq,
            length_ppq,
            asset_id,
        );
        track.clips.insert(new_clip.id.clone(), new_clip);

        tracks.insert(track.id.clone(), GeneratorTrack::AudioTrack(track.clone()));
        Some(track)
    }

    pub async fn add_clip_to_audio_track(&self, clip: ClipToInsert) -> Option<Clip> {
        let (asset_id, num_samples, clip_name) = self.ensure_audio_asset(clip.source_path).await?;
        let length_ppq = self.calc_clip_length_ppq(num_samples);

        let mut tracks = self.tracks.lock().unwrap();

        let track_id = match clip.track_id.as_deref() {
            Some(id) => id,
            None => {
                log_and_notify_error("Error trying to insert clip: missing trackId".to_string());
                return None;
            }
        };

        let maybe_target = tracks.get_mut(track_id);

        if let Some(target) = maybe_target {
            let audio = match target.as_audio_mut() {
                Some(a) => a,
                None => {
                    log_and_notify_error(format!(
                        "Error trying to insert clip: track is not an audio track: {}",
                        target.id()
                    ));
                    return None;
                }
            };

            let new_clip = Clip::new(
                audio.id.clone(),
                clip_name,
                0,
                clip.start_ppq,
                length_ppq,
                asset_id,
            );
            let clip_id = new_clip.id.clone();
            audio.clips.insert(clip_id.clone(), new_clip);
            return audio.clips.get(&clip_id).cloned();
        }

        log_and_notify_error(format!(
            "Error trying to insert clip: track not found: {track_id}"
        ));
        None
    }

    pub fn move_clip_in_audio_track(
        &self,
        track_id: Id,
        clip_id: Id,
        start_ppq: usize,
    ) -> Option<Clip> {
        let mut tracks = self.tracks.lock().unwrap();
        let t = tracks.get_mut(&track_id)?;
        let audio = t.as_audio_mut()?;
        let c = audio.clips.get_mut(&clip_id)?;
        c.start_ppq = start_ppq;
        Some(c.clone())
    }

    pub async fn assign_source_to_sampler_track(
        &self,
        track_id: Id,
        source_path: Option<String>,
    ) -> Option<Id> {
        let source_path = match source_path {
            Some(p) => p,
            None => return None,
        };

        let asset_id = if let Some(existing_id) = ASSET_POOL.audio.get_id_by_path(&source_path) {
            existing_id
        } else {
            let decode_result =
                async_runtime::spawn_blocking(move || decode_audio_file(source_path))
                    .await
                    .unwrap_or_else(|e| Err(Error::new(e)));

            match decode_result {
                Ok(decoded_audio_data) => ASSET_POOL.audio.add(decoded_audio_data),
                Err(e) => {
                    log_and_notify_error(format!("Error trying to load audio file: {e}"));
                    return None;
                }
            }
        };

        let mut tracks = self.tracks.lock().unwrap();
        let t = match tracks.get_mut(&track_id) {
            Some(t) => t,
            None => {
                log_and_notify_error(format!(
                    "Error trying to assign audio: track not found: {track_id}"
                ));
                return None;
            }
        };

        let sampler = match t.as_sampler_mut() {
            Some(s) => s,
            None => {
                log_and_notify_error(format!(
                    "Error trying to assign audio: track is not a sampler track: {track_id}"
                ));
                return None;
            }
        };

        sampler.source_id = Some(asset_id);
        sampler.source_id.clone()
    }

    pub fn delete_audio_track(&self, track_id: &str) {
        self.tracks.lock().unwrap().shift_remove(track_id);
    }

    pub fn with_tracks<R>(&self, f: impl FnOnce(&IndexMap<Id, GeneratorTrack>) -> R) -> R {
        let guard = self.tracks.lock().unwrap();
        f(&*guard)
    }

    pub fn stop(&self) {
        PREVIEW_MIXER.is_canceled.store(true, Ordering::SeqCst);
    }
}

pub static PROJECT_STATE: LazyLock<ProjectState> = LazyLock::new(|| ProjectState::new());
