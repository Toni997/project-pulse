use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;
use nanoid::nanoid;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    core::{constants::MASTER_TRACK_DEFAULT_NAME, types::Id},
    project::clip::Clip,
};

pub fn default_track_name(track_number: usize) -> String {
    format!("Track #{track_number}")
}

pub enum GeneratorTrack {
    AudioTrack(AudioTrack),
    SamplerTrack(SamplerTrack),
    // InstrumentTrack(InstrumentTrack)
}

impl GeneratorTrack {
    /// The fields every kind of generator track has.
    pub fn common(&self) -> &TrackCommon {
        match self {
            GeneratorTrack::AudioTrack(t) => t,
            GeneratorTrack::SamplerTrack(t) => t,
        }
    }

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

/// Fields shared by every track kind. Embedded via `#[serde(flatten)]` so the
/// JSON shape sent to the frontend is unchanged (fields stay top-level rather
/// than nested under `"common"`), and each track kind implements `Deref`/
/// `DerefMut` to it so existing call sites (`track.id`, `track.clips...`)
/// keep working without a `.common` prefix.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackCommon {
    pub id: Id,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub clips: IndexMap<Id, Clip>,
}

impl TrackCommon {
    fn new(name: impl Into<String>) -> Self {
        Self {
            id: nanoid!(),
            name: name.into(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            clips: IndexMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    #[serde(flatten)]
    common: TrackCommon,
}

impl Serialize for AudioTrack {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TrackRepr {
            common: &self.common,
            extra: NoExtra,
            kind: TrackKind::Audio,
        }
        .serialize(serializer)
    }
}

impl Deref for AudioTrack {
    type Target = TrackCommon;
    fn deref(&self) -> &TrackCommon {
        &self.common
    }
}

impl DerefMut for AudioTrack {
    fn deref_mut(&mut self) -> &mut TrackCommon {
        &mut self.common
    }
}

impl AudioTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            common: TrackCommon::new(name),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplerTrack {
    #[serde(flatten)]
    common: TrackCommon,
    pub source_id: Option<Id>,
}

impl Serialize for SamplerTrack {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TrackRepr {
            common: &self.common,
            extra: SamplerExtra {
                source_id: self.source_id.clone(),
            },
            kind: TrackKind::Sampler,
        }
        .serialize(serializer)
    }
}

impl Deref for SamplerTrack {
    type Target = TrackCommon;
    fn deref(&self) -> &TrackCommon {
        &self.common
    }
}

impl DerefMut for SamplerTrack {
    fn deref_mut(&mut self) -> &mut TrackCommon {
        &mut self.common
    }
}

impl SamplerTrack {
    pub fn new(name: impl Into<String>, source_id: Option<Id>) -> Self {
        Self {
            common: TrackCommon::new(name),
            source_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusTrack {
    #[serde(flatten)]
    common: TrackCommon,
}

impl Serialize for BusTrack {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TrackRepr {
            common: &self.common,
            extra: NoExtra,
            kind: TrackKind::Bus,
        }
        .serialize(serializer)
    }
}

impl Deref for BusTrack {
    type Target = TrackCommon;
    fn deref(&self) -> &TrackCommon {
        &self.common
    }
}

impl DerefMut for BusTrack {
    fn deref_mut(&mut self) -> &mut TrackCommon {
        &mut self.common
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterTrack {
    #[serde(flatten)]
    common: TrackCommon,
}

impl Serialize for MasterTrack {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TrackRepr {
            common: &self.common,
            extra: NoExtra,
            kind: TrackKind::Master,
        }
        .serialize(serializer)
    }
}

impl Deref for MasterTrack {
    type Target = TrackCommon;
    fn deref(&self) -> &TrackCommon {
        &self.common
    }
}

impl DerefMut for MasterTrack {
    fn deref_mut(&mut self) -> &mut TrackCommon {
        &mut self.common
    }
}

impl MasterTrack {
    pub fn new() -> Self {
        Self {
            common: TrackCommon::new(MASTER_TRACK_DEFAULT_NAME),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, strum_macros::Display)]
enum TrackKind {
    Audio,
    Sampler,
    Instrument,
    Bus,
    Master,
}

/// Shadow of a track's JSON shape used only to serialize a `kind` tag
/// alongside its flattened fields, without that tag existing as real data on
/// the track itself - the type (which struct, which `GeneratorTrack` variant)
/// already says what kind it is, `kind` only needs to exist on the wire.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackRepr<'a, E: Serialize> {
    #[serde(flatten)]
    common: &'a TrackCommon,
    #[serde(flatten)]
    extra: E,
    kind: TrackKind,
}

#[derive(Serialize)]
struct NoExtra;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SamplerExtra {
    source_id: Option<Id>,
}
