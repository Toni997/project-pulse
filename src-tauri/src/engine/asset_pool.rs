use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use nanoid::nanoid;
use parking_lot::RwLock;

use crate::{
    core::types::{EngineSampleFormat, Id},
    engine::decoder::DecodedAudioData,
};

/// Decoded audio: interleaved samples in the engine's sample format.
pub struct AudioSamples {
    data: Vec<EngineSampleFormat>,
}

impl AudioSamples {
    pub fn as_slice(&self) -> &[EngineSampleFormat] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub struct AudioMetaData {
    file_path: String,
    display_name: String,
    channels: usize,
    sample_rate: usize,
}

pub struct AudioAsset {
    samples: Arc<AudioSamples>,
    meta_data: ArcSwap<AudioMetaData>,
}

impl AudioAsset {
    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }

    pub fn display_name(&self) -> String {
        self.meta_data.load().display_name.clone()
    }
}

pub struct AudioStore {
    inner: RwLock<AudioStoreInner>,
}

impl AudioStore {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(AudioStoreInner {
                store: HashMap::new(),
                path_to_id: HashMap::new(),
            }),
        }
    }

    pub fn get_samples_by_id(&self, id: &str) -> Option<Arc<AudioSamples>> {
        Some(Arc::clone(&self.inner.read().store.get(id)?.samples))
    }

    /// Finds an asset by the path it was loaded from, taking the lock once.
    pub fn get_by_path(&self, path: &str) -> Option<(Id, Arc<AudioAsset>)> {
        let inner = self.inner.read();
        let id = inner.path_to_id.get(path)?;
        let asset = inner.store.get(id)?;
        Some((id.clone(), Arc::clone(asset)))
    }

    pub fn has_path(&self, path: &str) -> bool {
        self.inner.read().path_to_id.contains_key(path)
    }

    pub fn add(&self, decoded_audio_data: DecodedAudioData) -> Id {
        let DecodedAudioData {
            data,
            original_num_channels,
            original_sample_rate,
            file_path,
            file_name,
        } = decoded_audio_data;

        let mut inner = self.inner.write();
        if let Some(id) = inner.path_to_id.get(&file_path) {
            return id.clone();
        }

        let new_audio = Arc::new(AudioAsset {
            samples: Arc::new(AudioSamples { data }),
            meta_data: ArcSwap::new(Arc::new(AudioMetaData {
                file_path: file_path.clone(),
                display_name: file_name,
                channels: original_num_channels,
                sample_rate: original_sample_rate,
            })),
        });
        let id = nanoid!();

        inner.store.insert(id.clone(), new_audio);
        inner.path_to_id.insert(file_path, id.clone());
        id
    }

    pub fn remove(&self, id: &str) {
        let mut inner = self.inner.write();
        if let Some(asset) = inner.store.remove(id) {
            let path = asset.meta_data.load().file_path.clone();
            inner.path_to_id.remove(&path);
        }
    }
}

pub struct AssetPool {
    pub audio: AudioStore,
}

impl AssetPool {
    pub fn new() -> Self {
        Self {
            audio: AudioStore::new(),
        }
    }
}

struct AudioStoreInner {
    store: HashMap<Id, Arc<AudioAsset>>,
    path_to_id: HashMap<String, Id>,
}
