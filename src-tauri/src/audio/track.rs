use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::core::constants::MASTER_TRACK_DEFAULT_NAME;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAmount {
    pub bus_id: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrack {
    pub id: String,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub audio_file: String,
    pub sends: Vec<SendAmount>,
}

impl AudioTrack {
    pub fn new(name: &str) -> Self {
        Self {
            id: nanoid!(),
            name: name.to_string(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            audio_file: String::new(),
            sends: Vec::new(),
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
