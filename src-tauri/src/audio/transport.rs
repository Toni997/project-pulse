use std::sync::{
    atomic::{
        AtomicBool, AtomicUsize,
        Ordering::{self, SeqCst},
    },
    LazyLock, Mutex,
};

use anyhow::{anyhow, Context, Result};
use log::info;
use ringbuf::traits::{Observer, Producer};

use crate::{
    audio::{
        engine::AUDIO_ENGINE, preview_mixer::PREVIEW_MIXER,
        snapshot::project_snapshot::load_project_snapshot, thread_pool::AUDIO_WORKER_POOL,
    },
    core::constants::BUFFER_SIZE_DEFAULT,
};

pub struct Transport {
    pub is_playing: AtomicBool,
    pub position_ppq: AtomicUsize,
    pub loop_range_ppq: (AtomicUsize, AtomicUsize),
    pub position_samples: AtomicUsize,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            position_ppq: AtomicUsize::new(0),
            loop_range_ppq: (AtomicUsize::new(0), AtomicUsize::new(0)),
            position_samples: AtomicUsize::new(0),
        }
    }

    pub fn position_ppq(&self) -> usize {
        self.position_ppq.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        PREVIEW_MIXER.is_canceled.store(true, Ordering::SeqCst);
        self.is_playing
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.position_ppq.store(0, Ordering::SeqCst);
        self.position_samples.store(0, Ordering::SeqCst);
        AUDIO_WORKER_POOL.stop();
    }

    pub fn play(&self) -> Result<()> {
        info!(
            "Transport play, ppq {}, samples {}",
            self.position_ppq(),
            self.position_samples.load(SeqCst)
        );
        TRANSPORT
            .is_playing
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let mut engine_producer_guard = AUDIO_ENGINE
            .engine_producer
            .lock()
            .map_err(|_| anyhow!("Could not lock preview producer"))?;
        let engine_producer = engine_producer_guard
            .as_mut()
            .context("Preview producer missing or stream already started")?;
        let buffer_size = BUFFER_SIZE_DEFAULT as usize * AUDIO_ENGINE.num_channels();
        let snapshot = load_project_snapshot();
        let mut current_snapshot_version = snapshot.version.clone();
        let mut tracks = &snapshot.get_scheduler().tracks;
        // TODO find a way to not put these buffers inside a mutex
        let mut track_buffers = tracks
            .iter()
            .map(|_| Mutex::new(vec![0.0f32; buffer_size]))
            .collect::<Vec<_>>();
        let mut main_buffer = vec![0.0f32; buffer_size];
        AUDIO_WORKER_POOL.start();
        while TRANSPORT.is_playing.load(Ordering::SeqCst) {
            main_buffer.fill(0.0);
            let snapshot = load_project_snapshot();
            tracks = &snapshot.get_scheduler().tracks;
            if snapshot.version != current_snapshot_version {
                current_snapshot_version = snapshot.version.clone();
                track_buffers.truncate(tracks.len());
                while track_buffers.len() < tracks.len() {
                    track_buffers.push(Mutex::new(vec![0.0f32; buffer_size]));
                }
            }
            if engine_producer.vacant_len() < buffer_size {
                continue;
            }
            let position_samples = self.position_samples.load(Ordering::SeqCst);
            AUDIO_WORKER_POOL.run_parallel(tracks.len(), &|_, track_index| {
                let mut track_buffer = track_buffers[track_index].lock().unwrap();
                tracks[track_index].render(position_samples, &mut track_buffer, snapshot.clone());
            });

            for track_buffer in track_buffers.iter() {
                let track_buffer = track_buffer.lock().unwrap();
                for (main_sample, track_sample) in main_buffer.iter_mut().zip(track_buffer.iter()) {
                    *main_sample += *track_sample;
                }
            }
            engine_producer.push_slice(&main_buffer);
            self.position_samples
                .fetch_add(buffer_size, Ordering::SeqCst);
        }
        AUDIO_WORKER_POOL.stop();
        Ok(())
    }
}

pub static TRANSPORT: LazyLock<Transport> = LazyLock::new(|| Transport::new());
