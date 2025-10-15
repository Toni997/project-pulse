use crate::audio::mixer::AUDIO_MIXER;
use crate::core::constants::{DRIVER_BUFFER_SIZE, ENGINE_BUFFER_MUTLIPLIER};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, Host, Stream, StreamConfig};
use ringbuf::traits::Consumer;
use ringbuf::HeapRb;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

pub static mut ENGINE_BUFFER: LazyLock<HeapRb<f32>> = LazyLock::new(|| HeapRb::new(4096 as usize));

pub static mut PREVIEW_BUFFER: LazyLock<HeapRb<f32>> =
    LazyLock::new(|| HeapRb::new(44100 as usize));

pub struct AudioEngine {
    host: Host,
    device: Device,
    config: StreamConfig,
    stream: Option<Stream>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No output device available");
        let supported_config = device.default_output_config().unwrap();
        let mut config: StreamConfig = supported_config.config();
        config.buffer_size = BufferSize::Fixed(DRIVER_BUFFER_SIZE);

        let supported_configs_range = device.supported_output_configs().unwrap();
        println!("Supported output configs:");
        for config in supported_configs_range {
            let buffer_size = config.buffer_size();
            match buffer_size {
                cpal::SupportedBufferSize::Range { min, max } => {
                    println!(
                    "Channels: {:<2} | Sample rate: {:<6}-{:>6} | Buffer size range: {}–{} frames",
                    config.channels(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                    min,
                    max
                );
                }
                cpal::SupportedBufferSize::Unknown => {
                    println!(
                    "Channels: {:<2} | Sample rate: {:<6}-{:>6} | Buffer size: unknown (system decides)",
                    config.channels(),
                    config.min_sample_rate().0,
                    config.max_sample_rate().0,
                );
                }
            }
        }

        println!("Output stream sample rate: {}", config.sample_rate.0);
        println!("Output stream channels: {}", config.channels);

        Self {
            host,
            device,
            config,
            stream: None,
        }
    }

    pub fn start(&mut self) {
        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |output: &mut [f32], _| unsafe {
                    output.fill(0.0);
                    if AUDIO_MIXER.is_playing {
                        ENGINE_BUFFER.pop_slice(output);
                    }
                    if AUDIO_MIXER.is_preview_playing {
                        let mut preview_buf = vec![0.0f32; output.len()];
                        let read = PREVIEW_BUFFER.pop_slice(&mut preview_buf);
                        for (out, prev) in output.iter_mut().zip(preview_buf.iter()) {
                            *out += *prev;
                        }
                    }
                },
                move |err| {
                    eprintln!("Stream error: {}", err);
                },
                None,
            )
            .unwrap();

        stream.play().unwrap();
        self.stream = Some(stream);
    }
}

pub static mut AUDIO_ENGINE: LazyLock<AudioEngine> = LazyLock::new(|| AudioEngine::new());
