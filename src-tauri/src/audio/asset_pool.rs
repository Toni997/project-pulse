use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, RwLock},
};

use arc_swap::ArcSwap;
use nanoid::nanoid;

use crate::{
    audio::decoder::DecodedAudioData,
    core::types::{EngineSampleFormat, Id},
};

pub struct AudioPcmData {
    data: Vec<EngineSampleFormat>,
}

pub struct AudioMetaData {
    file_path: String,
    display_name: String,
    channels: usize,
    sample_rate: usize,
}

pub struct AudioAsset {
    pcmData: Arc<AudioPcmData>,
    metaData: ArcSwap<AudioMetaData>,
}

struct AudioStoreInner {
    store: HashMap<Id, Arc<AudioAsset>>,
    path_to_id: HashMap<String, Id>,
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

    pub fn get_pcm_by_id(&self, id: &str) -> Option<Arc<AudioPcmData>> {
        Some(self.inner.read().unwrap().store.get(id)?.pcmData.clone())
    }

    pub fn get_id_by_path(&self, path: &str) -> Option<Id> {
        self.inner.read().unwrap().path_to_id.get(path).cloned()
    }

    pub fn has_path(&self, path: &str) -> bool {
        self.inner.read().unwrap().path_to_id.contains_key(path)
    }

    pub fn add(&self, decoded_audio_data: DecodedAudioData) -> Id {
        let DecodedAudioData {
            data,
            original_num_channels,
            original_sample_rate,
            file_path,
            file_name,
        } = decoded_audio_data;

        let mut inner = self.inner.write().unwrap();
        if let Some(id) = inner.path_to_id.get(&file_path) {
            return id.clone();
        }

        let new_audio = Arc::new(AudioAsset {
            pcmData: Arc::new(AudioPcmData { data }),
            metaData: ArcSwap::new(Arc::new(AudioMetaData {
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
        let mut inner = self.inner.write().unwrap();
        if let Some(asset) = inner.store.remove(id) {
            let path = asset.metaData.load().file_path.clone();
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

pub static ASSET_POOL: LazyLock<AssetPool> = LazyLock::new(|| AssetPool::new());
