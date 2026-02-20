use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::core::constants::MASTER_TRACK_DEFAULT_NAME;

pub fn default_track_name(track_number: usize) -> String {
    format!("Track #{track_number}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: String,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub source_id: Option<String>,
}

impl AudioTrack {
    pub fn new(name: impl Into<String>, source_id: Option<String>) -> Self {
        Self {
            id: nanoid!(),
            name: name.into(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            source_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusTrack {
    pub id: String,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterTrack {
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
}

impl MasterTrack {
    pub fn default() -> Self {
        Self {
            name: String::from(MASTER_TRACK_DEFAULT_NAME),
            volume: 1.0,
            pan: 0.0,
            muted: false,
        }
    }
}
