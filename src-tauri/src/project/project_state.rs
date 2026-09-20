use anyhow::{anyhow, Result};
use atomic_float::AtomicF32;
use indexmap::IndexMap;
use log::info;
use parking_lot::Mutex;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU16, AtomicU64};
use std::sync::mpsc::Sender;

use crate::core::constants::{
    PPQ_DEFAULT, TEMPO_BPM_DEFAULT, TIME_SIGNATURE_DENOMINATOR_DEFAULT,
    TIME_SIGNATURE_NUMERATOR_DEFAULT,
};
use crate::core::rebuild_scope::RebuildScope;
use crate::core::types::Id;
use crate::project::clip::{Clip, NewClip};
use crate::project::track::{
    default_track_name, AudioTrack, BusTrack, GeneratorTrack, MasterTrack, SamplerTrack,
};

/// The editable project model - tracks, clips, tempo. Plain, synchronous data
/// mutation: no IO, no decoding, no UI notifications, no knowledge of audio
/// devices. Loading assets and resolving clip lengths is `ProjectEditor`'s job;
/// this type only receives finished values. Each successful mutation asks for
/// the snapshot parts it invalidated to be rebuilt via the channel given at
/// construction; whoever listens decides what to do about it.
pub struct ProjectState {
    ppq: AtomicU16,
    tempo_bpm: AtomicF32,
    time_signature: (u8, u8),
    master: Mutex<MasterTrack>,
    tracks: Mutex<IndexMap<Id, GeneratorTrack>>,
    buses: Mutex<IndexMap<Id, BusTrack>>,
    /// Bumped by every edit, inside the same critical section as the edit
    /// itself, so a reader that sees the same value before and after reading
    /// knows nothing was edited in between.
    version: AtomicU64,
    rebuild_requests: Sender<RebuildScope>,
}

impl ProjectState {
    pub fn new(rebuild_requests: Sender<RebuildScope>) -> Self {
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
            version: AtomicU64::new(0),
            rebuild_requests,
        }
    }

    pub fn ppq(&self) -> u16 {
        self.ppq.load(Ordering::SeqCst)
    }

    pub fn tempo_bpm(&self) -> f32 {
        self.tempo_bpm.load(Ordering::SeqCst)
    }

    pub fn add_sampler_track(&self, source_id: Option<Id>) -> SamplerTrack {
        let mut tracks = self.tracks.lock();
        let track = SamplerTrack::new(default_track_name(tracks.len() + 1), source_id);
        tracks.insert(
            track.id.clone(),
            GeneratorTrack::SamplerTrack(track.clone()),
        );
        self.record_change(RebuildScope::RENDER_GRAPH | RebuildScope::DATA_NODES);
        track
    }

    pub fn add_audio_track(&self) -> AudioTrack {
        info!("ProjectState: add_audio_track");
        let mut tracks = self.tracks.lock();
        let track = AudioTrack::new(default_track_name(tracks.len() + 1));
        tracks.insert(track.id.clone(), GeneratorTrack::AudioTrack(track.clone()));
        self.record_change(RebuildScope::RENDER_GRAPH | RebuildScope::DATA_NODES);
        track
    }

    pub fn add_audio_track_with_clip(&self, clip: NewClip) -> AudioTrack {
        let mut tracks = self.tracks.lock();
        let mut track = AudioTrack::new(default_track_name(tracks.len() + 1));

        let new_clip = clip.into_clip(track.id.clone());
        track.clips.insert(new_clip.id.clone(), new_clip);

        tracks.insert(track.id.clone(), GeneratorTrack::AudioTrack(track.clone()));
        self.record_change(RebuildScope::all());
        track
    }

    pub fn add_clip_to_audio_track(&self, track_id: &str, clip: NewClip) -> Result<Clip> {
        let mut tracks = self.tracks.lock();
        let target = tracks
            .get_mut(track_id)
            .ok_or_else(|| anyhow!("track not found: {track_id}"))?;
        let audio = target
            .as_audio_mut()
            .ok_or_else(|| anyhow!("track is not an audio track: {track_id}"))?;

        let new_clip = clip.into_clip(audio.id.clone());
        audio.clips.insert(new_clip.id.clone(), new_clip.clone());
        self.record_change(RebuildScope::DATA_NODES | RebuildScope::SCHEDULER);
        Ok(new_clip)
    }

    pub fn move_clip_in_audio_track(
        &self,
        track_id: &str,
        clip_id: &str,
        start_ppq: usize,
    ) -> Option<Clip> {
        let mut tracks = self.tracks.lock();
        let audio = tracks.get_mut(track_id)?.as_audio_mut()?;
        let clip = audio.clips.get_mut(clip_id)?;
        clip.start_ppq = start_ppq;
        let moved = clip.clone();
        self.record_change(RebuildScope::SCHEDULER);
        Some(moved)
    }

    pub fn assign_source_to_sampler_track(&self, track_id: &str, source_id: Id) -> Result<()> {
        let mut tracks = self.tracks.lock();
        let target = tracks
            .get_mut(track_id)
            .ok_or_else(|| anyhow!("track not found: {track_id}"))?;
        let sampler = target
            .as_sampler_mut()
            .ok_or_else(|| anyhow!("track is not a sampler track: {track_id}"))?;

        sampler.source_id = Some(source_id);
        self.record_change(RebuildScope::DATA_NODES);
        Ok(())
    }

    pub fn delete_audio_track(&self, track_id: &str) {
        info!("ProjectState: delete_audio_track: {}", track_id);
        let mut tracks = self.tracks.lock();
        if tracks.shift_remove(track_id).is_some() {
            self.record_change(RebuildScope::all());
        }
    }

    pub fn delete_clip_from_audio_track(&self, track_id: &str, clip_id: &str) {
        let mut tracks = self.tracks.lock();
        let Some(track) = tracks.get_mut(track_id) else {
            return;
        };
        let Some(audio_track) = track.as_audio_mut() else {
            return;
        };
        if audio_track.clips.shift_remove(clip_id).is_some() {
            self.record_change(RebuildScope::DATA_NODES | RebuildScope::SCHEDULER);
        }
    }

    /// Number of edits made so far. Read it before and after reading the
    /// project: if it changed, an edit happened in between and what was read
    /// may be a mix of old and new.
    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    pub fn with_master<R>(&self, f: impl FnOnce(&MasterTrack) -> R) -> R {
        f(&self.master.lock())
    }

    /// Ids of the generator tracks, in order.
    pub fn track_ids(&self) -> Vec<Id> {
        self.tracks.lock().keys().cloned().collect()
    }

    /// Runs `f` on one track. The lock is held only for that call, so edits
    /// are never kept waiting for long. `None` if there is no such track.
    pub fn with_track<R>(&self, id: &str, f: impl FnOnce(&GeneratorTrack) -> R) -> Option<R> {
        self.tracks.lock().get(id).map(f)
    }

    pub fn bus_ids(&self) -> Vec<Id> {
        self.buses.lock().keys().cloned().collect()
    }

    /// Like [`ProjectState::with_track`], for a bus.
    pub fn with_bus<R>(&self, id: &str, f: impl FnOnce(&BusTrack) -> R) -> Option<R> {
        self.buses.lock().get(id).map(f)
    }

    /// Must be called while still holding the lock that guards the edit, so the
    /// version changes together with the data.
    fn record_change(&self, scope: RebuildScope) {
        self.version.fetch_add(1, Ordering::Release);
        // The receiver only goes away at shutdown, when nobody needs it.
        let _ = self.rebuild_requests.send(scope);
    }
}

#[cfg(test)]
#[path = "../tests/project/project_state.rs"]
mod tests;
