use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock, Mutex,
    },
    thread,
    time::Duration,
};

use log::info;

use crate::{audio::decoder::stream_audio_file, core::notify::log_and_notify_error};

pub struct PreviewMixer {
    pub is_playing: AtomicBool,
    pub is_started: AtomicBool,
    pub is_queued: AtomicBool,
    pub is_canceled: AtomicBool,
    pub file_path: Mutex<String>,
}

impl PreviewMixer {
    pub fn new() -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            is_started: AtomicBool::new(false),
            is_queued: AtomicBool::new(false),
            is_canceled: AtomicBool::new(false),
            file_path: Mutex::new(String::default()),
        }
    }

    pub fn play(&self, file_path: &str) {
        info!("preview_audio_file");
        self.is_playing.store(false, Ordering::SeqCst);
        self.is_canceled.store(true, Ordering::SeqCst);
        {
            let mut file_path_guard = self.file_path.lock().unwrap();
            *file_path_guard = file_path.to_string();
        }
        if self.is_queued.load(Ordering::SeqCst) {
            info!("audio preview already queued");
        };
        self.is_queued.store(true, Ordering::SeqCst);
        while self.is_started.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }
        self.is_queued.store(false, Ordering::SeqCst);
        self.is_canceled.store(false, Ordering::SeqCst);

        tauri::async_runtime::spawn_blocking(move || {
            if let Err(e) = stream_audio_file() {
                log_and_notify_error(format!("Error trying to preview audio file: {e}"));
                PREVIEW_MIXER.is_playing.store(false, Ordering::SeqCst);
                PREVIEW_MIXER.is_started.store(false, Ordering::SeqCst);
                PREVIEW_MIXER.is_queued.store(false, Ordering::SeqCst);
            }
        });
    }
}

pub static PREVIEW_MIXER: LazyLock<PreviewMixer> = LazyLock::new(|| PreviewMixer::new());
