use crate::core::constants::{BUFFER_SIZE_DEFAULT, NUM_CHANNELS_DEFAULT};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    available_hosts, Device, FromSample, Host, HostId, Sample, SampleFormat, SampleRate,
    SizedSample, Stream, StreamConfig,
};
use ringbuf::traits::{Consumer, Observer, Split};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use serde::de::value::U64Deserializer;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

pub struct AudioEngine {
    host: Host,
    device: Device,
    config: StreamConfig,
    sample_format: SampleFormat,
    stream: Mutex<Option<Stream>>,
    pub engine_producer: Mutex<Option<HeapProd<f32>>>,
    pub preview_producer: Mutex<Option<HeapProd<f32>>>,
    engine_consumer: Mutex<Option<HeapCons<f32>>>,
    preview_consumer: Mutex<Option<HeapCons<f32>>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let hosts = available_hosts();
        println!("{hosts:?}");

        let host = cpal::default_host();
        // let host = if hosts.contains(&HostId::Asio) {
        //     println!("ASIO host available, attempting to use it.");
        //     cpal::host_from_id(HostId::Asio).unwrap_or_else(|_| cpal::default_host())
        // } else {
        //     cpal::default_host()
        // };

        let device = host
            .default_output_device()
            .expect("No output device available");

        println!("Host: {:?}", host.id());
        println!(
            "Output device: {}",
            device.name().unwrap_or("Unknown".to_string())
        );

        // Print all supported configs for debugging
        if let Ok(configs) = device.supported_output_configs() {
            println!("Supported configs:");
            for (i, config) in configs.enumerate() {
                println!("  {}: {:?}", i, config);
            }
        }

        // Try to find an f32 config, otherwise fall back to default (which might fail if not f32)
        let supported_config = device.default_output_config().unwrap();
        let sample_format = supported_config.sample_format();
        let mut config: StreamConfig = supported_config.config();
        config.buffer_size = cpal::BufferSize::Fixed(BUFFER_SIZE_DEFAULT as u32);
        // config.sample_rate = 48000;
        config.channels = NUM_CHANNELS_DEFAULT as u16;

        match supported_config.buffer_size() {
            cpal::SupportedBufferSize::Range { min, max } => {
                println!("Supported buffer size range: {} - {}", min, max);
            }
            cpal::SupportedBufferSize::Unknown => {
                println!("Supported buffer size: Unknown");
            }
        }

        println!(
            "AudioEngine initialized: {} Hz, {} channels, Format: {:?}",
            config.sample_rate, config.channels, sample_format
        );

        Self {
            host,
            device,
            config,
            sample_format,
            stream: Mutex::new(None),
            engine_producer: Mutex::new(None),
            preview_producer: Mutex::new(None),
            engine_consumer: Mutex::new(None),
            preview_consumer: Mutex::new(None),
        }
    }

    pub fn start(&self) {
        // Create and split the ring buffers
        let engine_rb = HeapRb::<f32>::new(4096);
        let (engine_prod, engine_cons) = engine_rb.split();

        let preview_rb = HeapRb::<f32>::new(44100);
        let (preview_prod, preview_cons) = preview_rb.split();

        *self.engine_producer.lock().unwrap() = Some(engine_prod);
        *self.preview_producer.lock().unwrap() = Some(preview_prod);
        *self.engine_consumer.lock().unwrap() = Some(engine_cons);
        *self.preview_consumer.lock().unwrap() = Some(preview_cons);

        match self.sample_format {
            cpal::SampleFormat::I8 => self.build_output_stream::<i8>(),
            cpal::SampleFormat::I16 => self.build_output_stream::<i16>(),
            cpal::SampleFormat::I24 => self.build_output_stream::<i32>(),
            cpal::SampleFormat::I32 => self.build_output_stream::<i32>(),
            cpal::SampleFormat::I64 => self.build_output_stream::<i64>(),
            cpal::SampleFormat::U8 => self.build_output_stream::<u8>(),
            cpal::SampleFormat::U16 => self.build_output_stream::<u16>(),
            cpal::SampleFormat::U24 => self.build_output_stream::<u32>(),
            cpal::SampleFormat::U32 => self.build_output_stream::<u32>(),
            cpal::SampleFormat::U64 => self.build_output_stream::<u64>(),
            cpal::SampleFormat::F32 => self.build_output_stream::<f32>(),
            cpal::SampleFormat::F64 => self.build_output_stream::<f64>(),
            _ => panic!("Unsupported sample format"),
        };
    }

    pub fn sample_rate(&self) -> usize {
        self.config.sample_rate as usize
    }

    pub fn num_channels(&self) -> usize {
        self.config.channels as usize
    }

    fn build_output_stream<SampleType>(&self)
    where
        SampleType: Sample + SizedSample + FromSample<f32> + Copy + Send + Debug + Display,
    {
        // Pre-allocate a scratch buffer to avoid allocation in the callback
        let mut mixer_temp_output = vec![0.0f32; 4096];
        let mut preview_temp_output = vec![0.0f32; 4096];

        let mut engine_consumer = self
            .engine_consumer
            .lock()
            .expect("Could not lock engine consumer")
            .take()
            .expect("Engine consumer missing or stream already started");
        let mut preview_consumer = self
            .preview_consumer
            .lock()
            .expect("Could not lock preview consumer")
            .take()
            .expect("Preview consumer missing or stream already started");

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
                        output[i] = SampleType::from_sample::<f32>(mixed);
                    }
                },
                move |err| eprintln!("Stream error: {}", err),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play stream");
        *self.stream.lock().unwrap() = Some(stream);
    }
}

pub static AUDIO_ENGINE: LazyLock<AudioEngine> = LazyLock::new(|| AudioEngine::new());
