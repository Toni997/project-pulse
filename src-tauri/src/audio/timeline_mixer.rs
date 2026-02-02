use atomic_float::AtomicF32;
use std::sync::atomic::AtomicUsize;
use std::sync::{atomic::Ordering, LazyLock, Mutex};

use crate::audio::preview_mixer::PREVIEW_MIXER;
use crate::audio::track::{AudioTrack, BusTrack, MasterTrack};
use crate::core::constants::{PPQ_DEFAULT, TEMPO_BPM_DEFAULT};

pub struct TimelineMixer {
    ppq: AtomicUsize,
    tempo_bpm: AtomicF32,
    master: Mutex<MasterTrack>,
    tracks: Mutex<Vec<AudioTrack>>,
    buses: Mutex<Vec<BusTrack>>,
}

impl TimelineMixer {
    pub fn new() -> Self {
        Self {
            ppq: AtomicUsize::new(PPQ_DEFAULT), // pulses per quarter note
            tempo_bpm: AtomicF32::new(TEMPO_BPM_DEFAULT),
            master: Mutex::new(MasterTrack::default()),
            tracks: Mutex::new(Vec::new()),
            buses: Mutex::new(Vec::new()),
        }
    }

    pub fn add_audio_track(&self) {
        let track = AudioTrack::new("New Track");
        self.tracks.lock().unwrap().push(track);
    }

    pub fn delete_audio_track(&self, track_id: &str) {
        self.tracks.lock().unwrap().retain(|t| t.id != track_id);
    }

    pub fn stop(&self) {
        PREVIEW_MIXER.is_canceled.store(true, Ordering::SeqCst);
    }
}

pub static TIMELINE_MIXER: LazyLock<TimelineMixer> = LazyLock::new(|| TimelineMixer::new());
