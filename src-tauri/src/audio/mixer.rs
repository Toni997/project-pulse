use log::{info, warn};
use std::time::Duration;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock, Mutex,
    },
    thread,
};
use tauri::AppHandle;
use tauri::Emitter;

use crate::{
    audio::{arrangement::Arrangement, decoder::stream_audio_file},
    core::constants::PREVIEW_AUDIO_ERROR_EVENT_NAME,
};

pub struct AudioMixer {
    arrangements: Vec<Arrangement>,
    active_playlist: usize,
    playlist_offset: usize,
    pub is_preview_playing: AtomicBool,
    pub is_playing: AtomicBool,
    pub is_preview_started: AtomicBool,
    pub is_preview_queued: AtomicBool,
    pub is_preview_canceled: AtomicBool,
    pub preview_file: Mutex<String>,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            arrangements: vec![Arrangement::new(String::from("New Arrangement"))],
            active_playlist: 0,
            playlist_offset: 0,
            is_preview_playing: AtomicBool::new(false),
            is_playing: AtomicBool::new(false),
            is_preview_started: AtomicBool::new(false),
            is_preview_queued: AtomicBool::new(false),
            is_preview_canceled: AtomicBool::new(false),
            preview_file: Mutex::new(String::from("")),
        }
    }
}

pub static AUDIO_MIXER: LazyLock<AudioMixer> = LazyLock::new(|| AudioMixer::new());

#[tauri::command]
pub fn preview_audio_file(app: AppHandle, file_path: String) {
    info!("preview_audio_file");
    AUDIO_MIXER
        .is_preview_playing
        .store(false, Ordering::SeqCst);
    AUDIO_MIXER
        .is_preview_canceled
        .store(true, Ordering::SeqCst);
    {
        let mut file_path_guard = AUDIO_MIXER.preview_file.lock().unwrap();
        *file_path_guard = file_path.to_string();
    }
    if AUDIO_MIXER.is_preview_queued.load(Ordering::SeqCst) {
        info!("audio preview already queued");
    };
    AUDIO_MIXER.is_preview_queued.store(true, Ordering::SeqCst);
    while AUDIO_MIXER.is_preview_started.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(50));
    }
    AUDIO_MIXER.is_preview_queued.store(false, Ordering::SeqCst);
    AUDIO_MIXER
        .is_preview_canceled
        .store(false, Ordering::SeqCst);

    tauri::async_runtime::spawn_blocking(move || {
        if let Err(e) = stream_audio_file() {
            warn!("Error trying to preview audio file: {:?}", e);
            AUDIO_MIXER
                .is_preview_playing
                .store(false, Ordering::SeqCst);
            AUDIO_MIXER
                .is_preview_started
                .store(false, Ordering::SeqCst);
            AUDIO_MIXER.is_preview_queued.store(false, Ordering::SeqCst);
            let _ = app.emit(
                PREVIEW_AUDIO_ERROR_EVENT_NAME,
                format!("Error trying to preview audio file: {e}"),
            );
        }
    });
}

#[tauri::command]
pub fn stop_audio() {
    AUDIO_MIXER
        .is_preview_canceled
        .store(true, Ordering::SeqCst);
}
