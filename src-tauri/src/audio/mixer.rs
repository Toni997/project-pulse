use std::sync::LazyLock;

use crate::audio::{arrangement::Arrangement, decoder::stream_audio_file};

pub struct AudioMixer {
    arrangements: Vec<Arrangement>,
    active_playlist: usize,
    playlist_offset: usize,
    pub is_preview_playing: bool,
    pub is_playing: bool,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            arrangements: vec![Arrangement::new(String::from("New Arrangement"))],
            active_playlist: 0,
            playlist_offset: 0,
            is_preview_playing: false,
            is_playing: false,
        }
    }

    fn play_preview(&mut self, path: &str) {
        // decode until preview buffer full
        // is_preview_playing to true
        // buffer will start being consumed on the engine immediately
    }
}

pub static mut AUDIO_MIXER: LazyLock<AudioMixer> = LazyLock::new(|| AudioMixer::new());

#[tauri::command]
pub fn play_audio_file(path: String) {
    tauri::async_runtime::spawn_blocking(move || stream_audio_file(&path));
}
