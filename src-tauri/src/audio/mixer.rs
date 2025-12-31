use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock, Mutex,
};
use std::time::Duration;

use ringbuf::traits::Consumer;
use tauri::{async_runtime::JoinHandle, AppHandle};

use crate::audio::{arrangement::Arrangement, decoder::stream_audio_file, engine::AUDIO_ENGINE};

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
pub async fn preview_audio_file(file_path: String) -> Result<(), String> {
    println!("preview_audio_file");
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
        println!("already queued");
        return Ok(());
    };
    AUDIO_MIXER.is_preview_queued.store(true, Ordering::SeqCst);
    loop {
        if AUDIO_MIXER.is_preview_started.load(Ordering::SeqCst) {
            // println!("looping");
            // tokio::time::sleep(Duration::from_millis(50)).await;
            continue;
        };
        AUDIO_MIXER.is_preview_queued.store(false, Ordering::SeqCst);
        AUDIO_MIXER
            .is_preview_canceled
            .store(false, Ordering::SeqCst);

        tauri::async_runtime::spawn_blocking(move || stream_audio_file())
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| {
                println!("Error trying to preview audio file: {}", e);
                AUDIO_MIXER
                    .is_preview_playing
                    .store(false, Ordering::SeqCst);
                AUDIO_MIXER
                    .is_preview_started
                    .store(false, Ordering::SeqCst);
                AUDIO_MIXER.is_preview_queued.store(false, Ordering::SeqCst);
                e.to_string()
            })?;
        break;
    }
    Ok(())
}
