use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, LazyLock,
};

use anyhow::{anyhow, Context, Result};
use ringbuf::traits::{Observer, Producer};

use crate::{
    audio::{
        engine::AUDIO_ENGINE,
        project_state::PROJECT_STATE,
        snapshot::{
            project_snapshot::{load_project_snapshot, PROJECT_SNAPSHOT},
            scheduler::SchedulerAudioTrack,
        },
        track,
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

    pub fn play(&self) -> Result<()> {
        let new_position_samples = self.position_ppq() * PROJECT_STATE.ppq() as usize;
        self.position_samples
            .store(new_position_samples, Ordering::SeqCst);
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
        let mut track_buffers = vec![vec![0.0f32; buffer_size]; tracks.len()];
        let mut current_active_clip_indexes = vec![0; tracks.len()];
        let mut main_buffer = vec![0.0f32; buffer_size];
        while TRANSPORT.is_playing.load(Ordering::SeqCst) {
            main_buffer.fill(0.0);
            let snapshot = load_project_snapshot();
            println!("Transport: play LOOP, {}", snapshot.version);
            tracks = &snapshot.get_scheduler().tracks;
            if snapshot.version != current_snapshot_version {
                current_snapshot_version = snapshot.version.clone();
                track_buffers.resize(tracks.len(), vec![0.0f32; buffer_size]);
                current_active_clip_indexes.resize(tracks.len(), 0);
            }
            if engine_producer.vacant_len() < buffer_size {
                continue;
            }
            // TODO use thread pool here
            for (index, track) in tracks.iter().enumerate() {
                track.fill_with_active_samples(
                    self.position_samples.load(Ordering::SeqCst),
                    &mut current_active_clip_indexes[index],
                    &mut track_buffers[index],
                    snapshot.clone(),
                );
            }
            for track_buffer in track_buffers.iter() {
                for (main_sample, track_sample) in main_buffer.iter_mut().zip(track_buffer.iter()) {
                    *main_sample += *track_sample;
                }
            }
            engine_producer.push_slice(&main_buffer);
        }
        Ok(())
    }
}

pub static TRANSPORT: LazyLock<Transport> = LazyLock::new(|| Transport::new());
