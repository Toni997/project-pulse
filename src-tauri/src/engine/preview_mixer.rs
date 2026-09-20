use std::{
    ops::ControlFlow,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use anyhow::Result;
use log::info;
use parking_lot::Mutex;
use ringbuf::{
    traits::{Observer, Producer},
    HeapProd,
};

use crate::{
    core::notify::log_and_notify_error,
    engine::{
        audio_reader::AudioFileReader, device::AudioEngine,
        resampler_builder::create_preview_resampler,
    },
};

/// How long to wait for the audio device to drain the ring buffer before
/// checking again.
const BUFFER_FULL_WAIT: Duration = Duration::from_millis(2);

pub struct PreviewMixer {
    pub is_playing: AtomicBool,
    pub is_started: AtomicBool,
    pub is_queued: AtomicBool,
    pub is_canceled: AtomicBool,
    pub file_path: Mutex<String>,
    producer: Mutex<HeapProd<f32>>,
    audio_engine: Arc<AudioEngine>,
}

impl PreviewMixer {
    pub fn new(audio_engine: Arc<AudioEngine>, producer: HeapProd<f32>) -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            is_started: AtomicBool::new(false),
            is_queued: AtomicBool::new(false),
            is_canceled: AtomicBool::new(false),
            file_path: Mutex::new(String::default()),
            producer: Mutex::new(producer),
            audio_engine,
        }
    }

    pub fn play(self: &Arc<Self>, file_path: &str) {
        info!("preview_audio_file");
        self.is_playing.store(false, Ordering::SeqCst);
        self.is_canceled.store(true, Ordering::SeqCst);
        {
            let mut file_path_guard = self.file_path.lock();
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

        let this = Arc::clone(self);
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(e) = this.stream_file() {
                log_and_notify_error(&format!("Error trying to preview audio file: {e}"));
                this.is_playing.store(false, Ordering::SeqCst);
                this.is_started.store(false, Ordering::SeqCst);
                this.is_queued.store(false, Ordering::SeqCst);
            }
        });
    }

    /// Decodes the queued file and feeds it to the output ring buffer as it
    /// drains, stopping early if the preview is canceled.
    fn stream_file(&self) -> Result<()> {
        self.is_started.store(true, Ordering::SeqCst);
        let file_path = self.file_path.lock().clone();
        info!("starting streaming {}", file_path);
        self.is_playing.store(false, Ordering::SeqCst);

        let mut producer = self.producer.lock();
        let reader =
            AudioFileReader::open(&file_path, self.audio_engine.sample_rate(), |source_rate| {
                create_preview_resampler(source_rate, &self.audio_engine)
            })?;

        self.is_playing.store(true, Ordering::SeqCst);
        reader.for_each_chunk(|chunk| {
            let mut remaining = chunk;
            while !remaining.is_empty() {
                if self.is_canceled.load(Ordering::SeqCst) {
                    return ControlFlow::Break(());
                }
                let writable = producer.vacant_len().min(remaining.len());
                if writable == 0 {
                    thread::sleep(BUFFER_FULL_WAIT);
                    continue;
                }
                producer.push_slice(&remaining[..writable]);
                remaining = &remaining[writable..];
            }
            ControlFlow::Continue(())
        })?;

        self.is_playing.store(false, Ordering::SeqCst);
        self.is_started.store(false, Ordering::SeqCst);
        Ok(())
    }
}
