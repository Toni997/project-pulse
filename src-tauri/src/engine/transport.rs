use std::sync::{
    atomic::{
        AtomicBool, AtomicUsize,
        Ordering::{self, SeqCst},
    },
    Arc,
};

use anyhow::{Context, Result};
use log::info;
use parking_lot::Mutex;
use ringbuf::{
    traits::{Observer, Producer},
    HeapProd,
};

use crate::{
    core::constants::BUFFER_SIZE_DEFAULT,
    engine::{
        device::AudioEngine, mixing_thread_pool::MixingThreadPool, preview_mixer::PreviewMixer,
    },
    snapshot::publisher::SnapshotPublisher,
};

pub struct Transport {
    pub is_playing: AtomicBool,
    pub position_ppq: AtomicUsize,
    pub loop_range_ppq: (AtomicUsize, AtomicUsize),
    pub position_samples: AtomicUsize,
    audio_engine: Arc<AudioEngine>,
    preview_mixer: Arc<PreviewMixer>,
    mixing_thread_pool: Arc<MixingThreadPool>,
    snapshot: Arc<SnapshotPublisher>,
    // Held for the whole play loop, so it also serves as the "already playing" guard.
    engine_producer: Mutex<HeapProd<f32>>,
}

impl Transport {
    pub fn new(
        audio_engine: Arc<AudioEngine>,
        preview_mixer: Arc<PreviewMixer>,
        mixing_thread_pool: Arc<MixingThreadPool>,
        snapshot: Arc<SnapshotPublisher>,
        engine_producer: HeapProd<f32>,
    ) -> Self {
        Self {
            is_playing: AtomicBool::new(false),
            position_ppq: AtomicUsize::new(0),
            loop_range_ppq: (AtomicUsize::new(0), AtomicUsize::new(0)),
            position_samples: AtomicUsize::new(0),
            audio_engine,
            preview_mixer,
            mixing_thread_pool,
            snapshot,
            engine_producer: Mutex::new(engine_producer),
        }
    }

    pub fn position_ppq(&self) -> usize {
        self.position_ppq.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.preview_mixer.is_canceled.store(true, Ordering::SeqCst);
        self.is_playing.store(false, Ordering::SeqCst);
        self.position_ppq.store(0, Ordering::SeqCst);
        self.position_samples.store(0, Ordering::SeqCst);
        self.mixing_thread_pool.stop();
    }

    pub fn play(&self) -> Result<()> {
        let mut engine_producer = self
            .engine_producer
            .try_lock()
            .context("Transport is already playing")?;
        info!(
            "Transport play, ppq {}, samples {}",
            self.position_ppq(),
            self.position_samples.load(SeqCst)
        );
        self.is_playing.store(true, Ordering::SeqCst);
        let buffer_size = BUFFER_SIZE_DEFAULT as usize * self.audio_engine.num_channels();
        let snapshot = self.snapshot.load();
        let mut current_snapshot_version = snapshot.version().clone();
        let mut tracks = snapshot.scheduler().tracks();
        // TODO find a way to not put these buffers inside a mutex
        let mut track_buffers = tracks
            .iter()
            .map(|_| Mutex::new(vec![0.0f32; buffer_size]))
            .collect::<Vec<_>>();
        let mut main_buffer = vec![0.0f32; buffer_size];
        self.mixing_thread_pool.start();
        while self.is_playing.load(Ordering::SeqCst) {
            main_buffer.fill(0.0);
            let snapshot = self.snapshot.load();
            tracks = snapshot.scheduler().tracks();
            if snapshot.version() != &current_snapshot_version {
                current_snapshot_version = snapshot.version().clone();
                track_buffers.truncate(tracks.len());
                while track_buffers.len() < tracks.len() {
                    track_buffers.push(Mutex::new(vec![0.0f32; buffer_size]));
                }
            }
            if engine_producer.vacant_len() < buffer_size {
                continue;
            }
            let position_samples = self.position_samples.load(Ordering::SeqCst);
            self.mixing_thread_pool
                .run_parallel(tracks.len(), &|_, track_index| {
                    let mut track_buffer = track_buffers[track_index].lock();
                    tracks[track_index].render(
                        position_samples,
                        &mut track_buffer,
                        snapshot.data_nodes(),
                    );
                });

            for track_buffer in track_buffers.iter() {
                let track_buffer = track_buffer.lock();
                for (main_sample, track_sample) in main_buffer.iter_mut().zip(track_buffer.iter()) {
                    *main_sample += *track_sample;
                }
            }
            engine_producer.push_slice(&main_buffer);
            self.position_samples
                .fetch_add(buffer_size, Ordering::SeqCst);
        }
        self.mixing_thread_pool.stop();
        Ok(())
    }
}
