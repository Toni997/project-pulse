use std::sync::LazyLock;

use crate::audio::arrangement::Arrangement;

pub struct AudioMixer {
    arrangements: Vec<Arrangement>,
    active_playlist: usize,
    playlist_offset: usize,
    preview_sample: Option<Vec<f32>>,
    preview_offset: usize,
    pub is_preview_playing: bool,
    pub is_playing: bool,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            arrangements: vec![Arrangement::new(String::from("New Arrangement"))],
            active_playlist: 0,
            playlist_offset: 0,
            preview_sample: None,
            preview_offset: 0,
            is_preview_playing: false,
            is_playing: false,
        }
    }

    pub fn play_preview(audio_file_path: String) {}
}

pub static mut AUDIO_MIXER: LazyLock<AudioMixer> = LazyLock::new(|| AudioMixer::new());
