use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::core::types::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: Id,
    pub track_id: Id,
    pub name: String,
    pub source_offset_samples: usize,
    pub start_ppq: usize,
    pub length_ppq: usize,
    pub source_id: Id,
}

impl Clip {
    pub fn new(
        track_id: Id,
        name: String,
        source_offset_samples: usize,
        start_ppq: usize,
        length_ppq: usize,
        source_id: Id,
    ) -> Self {
        Self {
            id: nanoid!(),
            track_id,
            name,
            source_offset_samples,
            start_ppq,
            length_ppq,
            source_id,
        }
    }
}

/// A clip ready to be placed on a track: source already loaded, length
/// already resolved.
pub struct NewClip {
    pub name: String,
    pub start_ppq: usize,
    pub length_ppq: usize,
    pub source_id: Id,
}

impl NewClip {
    pub fn into_clip(self, track_id: Id) -> Clip {
        Clip::new(
            track_id,
            self.name,
            0,
            self.start_ppq,
            self.length_ppq,
            self.source_id,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipToInsert {
    pub track_id: Option<Id>,
    pub start_ppq: usize,
    pub source_path: String,
}
