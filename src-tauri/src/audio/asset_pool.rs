use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;

pub struct AudioPcmData {
    data: Vec<f32>,
    channels: usize,
}

pub struct AudioMetaData {
    display_name: String,
    original_sample_rate: usize,
}

pub struct AudioAsset {
    pcmData: Arc<AudioPcmData>,
    metaData: ArcSwap<AudioMetaData>,
}

pub struct AssetPool {
    audio: HashMap<String, Arc<AudioAsset>>,
}

impl AssetPool {
    pub fn new() -> Self {
        Self {
            audio: HashMap::new(),
        }
    }

    pub fn get_audio_pcm_by_id(&self, id: &str) -> Option<Arc<AudioPcmData>> {
        Some(self.audio.get(id)?.pcmData.clone())
    }

    // pub fn add_audio(&mut self, path: String, data: Vec<f32>) {
    //     let new_audio = AudioAsset {
    //         pcmData: Arc::new(AudioPcmData {
    //             data,
    //             channels: 2,
    //         }),
    //         metaData: ArcSwap::new(AudioMetaData {
    //     }

    //     self.audio.insert(path, data);
    // }

    pub fn remove_audio(&mut self, path: &str) {
        self.audio.remove(path);
    }
}
