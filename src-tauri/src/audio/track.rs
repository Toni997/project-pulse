use indexmap::IndexMap;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    audio::clip::Clip,
    core::{constants::MASTER_TRACK_DEFAULT_NAME, types::Id},
};

pub fn default_track_name(track_number: usize) -> String {
    format!("Track #{track_number}")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, strum_macros::Display)]
enum TrackKind {
    Audio,
    Sampler,
    Instrument,
    Bus,
    Master,
}

pub enum GeneratorTrack {
    AudioTrack(AudioTrack),
    SamplerTrack(SamplerTrack),
    // InstrumentTrack(InstrumentTrack)
}

impl GeneratorTrack {
    pub fn id(&self) -> &str {
        match self {
            GeneratorTrack::AudioTrack(t) => &t.id,
            GeneratorTrack::SamplerTrack(t) => &t.id,
        }
    }

    pub fn as_sampler_mut(&mut self) -> Option<&mut SamplerTrack> {
        match self {
            GeneratorTrack::SamplerTrack(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_audio_mut(&mut self) -> Option<&mut AudioTrack> {
        match self {
            GeneratorTrack::AudioTrack(t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: Id,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub clips: IndexMap<Id, Clip>,
    kind: TrackKind,
}

impl AudioTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: nanoid!(),
            name: name.into(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            clips: IndexMap::new(),
            kind: TrackKind::Audio,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplerTrack {
    pub id: Id,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub source_id: Option<Id>,
    pub clips: IndexMap<Id, Clip>,
    kind: TrackKind,
}

impl SamplerTrack {
    pub fn new(name: impl Into<String>, source_id: Option<Id>) -> Self {
        Self {
            id: nanoid!(),
            name: name.into(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            source_id,
            clips: IndexMap::new(),
            kind: TrackKind::Sampler,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusTrack {
    pub id: Id,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub clips: IndexMap<Id, Clip>,
    kind: TrackKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterTrack {
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub clips: IndexMap<Id, Clip>,
    kind: TrackKind,
}

impl MasterTrack {
    pub fn new() -> Self {
        Self {
            name: String::from(MASTER_TRACK_DEFAULT_NAME),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            clips: IndexMap::new(),
            kind: TrackKind::Master,
        }
    }
}
