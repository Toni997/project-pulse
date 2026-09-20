use crate::core::constants::{BUFFER_SIZE_DEFAULT, STEREO_NUM_CHANNELS};
use crate::core::types::EngineSampleFormat;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    available_hosts, Device, FromSample, Host, Sample, SampleFormat, SizedSample, Stream,
    StreamConfig,
};
use log::{error, info};
use parking_lot::Mutex;
use ringbuf::traits::{Consumer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use std::fmt::{Debug, Display};

pub struct AudioEngine {
    host: Host,
    device: Device,
    config: StreamConfig,
    sample_format: SampleFormat,
    stream: Mutex<Option<Stream>>,
}

/// Writer ends of the ring buffers feeding the output stream. Each end is
/// handed to whoever owns that audio source.
pub struct EngineProducers {
    pub engine: HeapProd<f32>,
    pub preview: HeapProd<f32>,
}

/// Reader ends, consumed by the output stream callback in `AudioEngine::start`.
pub struct EngineConsumers {
    engine: HeapCons<f32>,
    preview: HeapCons<f32>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let hosts = available_hosts();
        info!("{hosts:?}");

        let host = cpal::default_host();
        // let host = if hosts.contains(&HostId::Asio) {
        //     info!("ASIO host available, attempting to use it.");
        //     cpal::host_from_id(HostId::Asio).unwrap_or_else(|_| cpal::default_host())
        // } else {
        //     cpal::default_host()
        // };

        let device = host
            .default_output_device()
            .expect("No output device available");

        info!("Host: {:?}", host.id());
        info!(
            "Output device: {}",
            device.name().unwrap_or("Unknown".to_string())
        );

        // Print all supported configs for debugging
        if let Ok(configs) = device.supported_output_configs() {
            info!("Supported configs:");
            for (i, config) in configs.enumerate() {
                info!("  {}: {:?}", i, config);
            }
        }

        // Try to find an f32 config, otherwise fall back to default (which might fail if not f32)
        let supported_config = device.default_output_config().unwrap();
        let sample_format = supported_config.sample_format();
        let mut config: StreamConfig = supported_config.config();
        config.buffer_size = cpal::BufferSize::Fixed(BUFFER_SIZE_DEFAULT as u32);
        config.channels = STEREO_NUM_CHANNELS as u16;

        match supported_config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => {
                info!("Supported buffer size range: {} - {}", min, max);
            }
            cpal::SupportedBufferSize::Unknown => {
                info!("Supported buffer size: Unknown");
            }
        }

        info!(
            "AudioEngine initialized: {} Hz, {} channels, Format: {:?}",
            config.sample_rate, config.channels, sample_format
        );

        Self {
            host,
            device,
            config,
            sample_format,
            stream: Mutex::new(None),
        }
    }

    /// Sized from this device's channel count, so it can only be called once
    /// the engine exists.
    pub fn create_ring_buffers(&self) -> (EngineProducers, EngineConsumers) {
        // Double the buffer size for safety
        let buffer_multiplier = 2;
        let (engine_prod, engine_cons) = HeapRb::<f32>::new(
            BUFFER_SIZE_DEFAULT as usize * self.num_channels() * buffer_multiplier,
        )
        .split();
        let (preview_prod, preview_cons) = HeapRb::<f32>::new(44100).split();

        (
            EngineProducers {
                engine: engine_prod,
                preview: preview_prod,
            },
            EngineConsumers {
                engine: engine_cons,
                preview: preview_cons,
            },
        )
    }

    pub fn start(&self, consumers: EngineConsumers) {
        match self.sample_format {
            cpal::SampleFormat::I8 => self.build_output_stream::<i8>(consumers),
            cpal::SampleFormat::I16 => self.build_output_stream::<i16>(consumers),
            cpal::SampleFormat::I24 => self.build_output_stream::<i32>(consumers),
            cpal::SampleFormat::I32 => self.build_output_stream::<i32>(consumers),
            cpal::SampleFormat::I64 => self.build_output_stream::<i64>(consumers),
            cpal::SampleFormat::U8 => self.build_output_stream::<u8>(consumers),
            cpal::SampleFormat::U16 => self.build_output_stream::<u16>(consumers),
            cpal::SampleFormat::U24 => self.build_output_stream::<u32>(consumers),
            cpal::SampleFormat::U32 => self.build_output_stream::<u32>(consumers),
            cpal::SampleFormat::U64 => self.build_output_stream::<u64>(consumers),
            cpal::SampleFormat::F32 => self.build_output_stream::<f32>(consumers),
            cpal::SampleFormat::F64 => self.build_output_stream::<f64>(consumers),
            _ => panic!("Unsupported sample format"),
        };
    }

    pub fn sample_rate(&self) -> usize {
        self.config.sample_rate as usize
    }

    pub fn num_channels(&self) -> usize {
        self.config.channels as usize
    }

    fn build_output_stream<SampleType>(&self, consumers: EngineConsumers)
    where
        SampleType: Sample + SizedSample + FromSample<f32> + Copy + Send + Debug + Display,
    {
        // Pre-allocate a scratch buffer to avoid allocation in the callback
        let mut mixer_temp_output = vec![0.0f32; 4096];
        let mut preview_temp_output = vec![0.0f32; 4096];

        let EngineConsumers {
            engine: mut engine_consumer,
            preview: mut preview_consumer,
        } = consumers;

        let stream = self
            .device
            .build_output_stream(
                &self.config,
                move |output: &mut [SampleType], _| {
                    if mixer_temp_output.len() < output.len() {
                        mixer_temp_output.resize(output.len(), 0.0);
                    }
                    if preview_temp_output.len() < output.len() {
                        preview_temp_output.resize(output.len(), 0.0);
                    }

                    let engine_slice = &mut mixer_temp_output[..output.len()];
                    let preview_slice = &mut preview_temp_output[..output.len()];

                    engine_slice.fill(0.0);
                    preview_slice.fill(0.0);

                    engine_consumer.pop_slice(engine_slice);
                    preview_consumer.pop_slice(preview_slice);

                    for i in 0..output.len() {
                        let mixed = engine_slice[i] + preview_slice[i];
                        output[i] = SampleType::from_sample::<EngineSampleFormat>(mixed);
                    }
                },
                move |err| error!("Stream error: {}", err),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play stream");
        *self.stream.lock() = Some(stream);
    }
}
