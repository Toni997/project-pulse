use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock, Mutex,
};

use ringbuf::traits::Consumer;
use tauri::async_runtime::JoinHandle;

use crate::audio::{arrangement::Arrangement, decoder::stream_audio_file, engine::PREVIEW_BUFFER};

pub struct AudioMixer {
    arrangements: Vec<Arrangement>,
    active_playlist: usize,
    playlist_offset: usize,
    pub is_preview_playing: AtomicBool,
    pub is_playing: bool,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            arrangements: vec![Arrangement::new(String::from("New Arrangement"))],
            active_playlist: 0,
            playlist_offset: 0,
            is_preview_playing: AtomicBool::new(false),
            is_playing: false,
        }
    }
}

pub static AUDIO_MIXER: LazyLock<AudioMixer> = LazyLock::new(|| AudioMixer::new());

pub static PREVIEW_AUDIO_RUNNING: AtomicBool = AtomicBool::new(false);
pub static PREVIEW_ALREADY_QUEUED: AtomicBool = AtomicBool::new(false);
pub static PREVIEW_CANCEL_FLAG: AtomicBool = AtomicBool::new(false);
pub static PREVIEW_AUDIO_PATH: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new(String::from("")));

#[tauri::command]
pub fn preview_audio_file(file_path: String) {
    AUDIO_MIXER
        .is_preview_playing
        .store(false, Ordering::SeqCst);
    PREVIEW_CANCEL_FLAG.store(true, Ordering::SeqCst);
    let mut file_path_guard = PREVIEW_AUDIO_PATH.lock().unwrap(); // lock for mutable access
    *file_path_guard = file_path.to_string();
    if PREVIEW_ALREADY_QUEUED.load(Ordering::SeqCst) {
        return;
    };
    PREVIEW_ALREADY_QUEUED.store(true, Ordering::SeqCst);
    loop {
        if PREVIEW_AUDIO_RUNNING.load(Ordering::SeqCst) {
            continue;
        };
        PREVIEW_ALREADY_QUEUED.store(false, Ordering::SeqCst);
        PREVIEW_CANCEL_FLAG.store(false, Ordering::SeqCst);
        tauri::async_runtime::spawn_blocking(move || {
            stream_audio_file();
        });
        break;
    }
}
