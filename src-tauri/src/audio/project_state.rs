use anyhow::Error;
use arc_swap::ArcSwap;
use atomic_float::AtomicF32;
use log::warn;
use serde::Serialize;
use std::sync::atomic::{AtomicU16, AtomicUsize};
use std::sync::{atomic::Ordering, LazyLock, Mutex};
use tauri::async_runtime;
use tauri::Emitter;

use crate::app_handle;
use crate::audio::asset_pool::ASSET_POOL;
use crate::audio::decoder::decode_audio_file;
use crate::audio::preview_mixer::PREVIEW_MIXER;
use crate::audio::track::{default_track_name, AudioTrack, BusTrack, MasterTrack};
use crate::core::constants::{
    NOTIFICATION_ERROR_EVENT, PPQ_DEFAULT, TEMPO_BPM_DEFAULT, TIME_SIGNATURE_DENOMINATOR_DEFAULT,
    TIME_SIGNATURE_NUMERATOR_DEFAULT,
};
use crate::core::types::Id;

pub struct ProjectState {
    ppq: AtomicU16,
    tempo_bpm: AtomicF32,
    time_signature: (u8, u8),
    master: Mutex<MasterTrack>,
    tracks: Mutex<Vec<AudioTrack>>,
    buses: Mutex<Vec<BusTrack>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            ppq: AtomicU16::new(PPQ_DEFAULT),
            tempo_bpm: AtomicF32::new(TEMPO_BPM_DEFAULT),
            time_signature: (
                TIME_SIGNATURE_NUMERATOR_DEFAULT,
                TIME_SIGNATURE_DENOMINATOR_DEFAULT,
            ),
            master: Mutex::new(MasterTrack::default()),
            tracks: Mutex::new(Vec::new()),
            buses: Mutex::new(Vec::new()),
        }
    }

    pub async fn add_audio_track(&self, source_path: Option<String>) -> Option<AudioTrack> {
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
                        warn!("Error trying to load audio file: {:?}", e);
                        let _ = app_handle().emit(
                            NOTIFICATION_ERROR_EVENT,
                            format!("Error trying to load audio file: {e}"),
                        );
                        return None;
                    }
                }
            }
        } else {
            None
        };

        let mut tracks = self.tracks.lock().unwrap();
        let track = AudioTrack::new(default_track_name(tracks.len() + 1), asset_id);
        tracks.push(track.clone());
        Some(track)
    }

    pub async fn assign_audio_to_track(
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
                    warn!("Error trying to load audio file: {:?}", e);
                    let _ = app_handle().emit(
                        NOTIFICATION_ERROR_EVENT,
                        format!("Error trying to load audio file: {e}"),
                    );
                    return None;
                }
            }
        };

        let mut tracks = self.tracks.lock().unwrap();
        let track = match tracks.iter_mut().find(|t| t.id == track_id) {
            Some(t) => t,
            None => {
                warn!(
                    "Error trying to assign audio: track not found: {}",
                    track_id
                );
                let _ = app_handle().emit(
                    NOTIFICATION_ERROR_EVENT,
                    format!("Error trying to assign audio: track not found"),
                );
                return None;
            }
        };

        track.source_id = Some(asset_id);
        track.source_id.clone()
    }

    pub fn delete_audio_track(&self, track_id: &str) {
        self.tracks.lock().unwrap().retain(|t| t.id != track_id);
    }

    pub fn stop(&self) {
        PREVIEW_MIXER.is_canceled.store(true, Ordering::SeqCst);
    }
}

pub static PROJECT_STATE: LazyLock<ProjectState> = LazyLock::new(|| ProjectState::new());
