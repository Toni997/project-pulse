use anyhow::{Context, Result};
use core::num;
use ringbuf::traits::{Consumer, Observer, Producer, RingBuffer};
use rubato::Resampler;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, RwLock};
use std::{f32, fs};
use symphonia::core::audio::{
    AudioBuffer, AudioBufferRef, Channels, SampleBuffer, Signal, SignalSpec,
};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::conv::IntoSample;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::{Hint, ProbeResult};
use symphonia::core::sample;
use symphonia::default::{get_codecs, get_probe};
use tauri::{AppHandle, Emitter};

use crate::audio::engine::AUDIO_ENGINE;
use crate::audio::mixer::AUDIO_MIXER;
use crate::audio::resampler::{resampler, resampler_2};

const MINIMUM_BUFFER_SPACE: usize = 5000;

pub static DECODED_AUDIO_CACHE: LazyLock<RwLock<HashMap<String, Vec<f32>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn get_media_source_stream_for_caching(file_path: &str) -> Result<MediaSourceStream> {
    let file_bytes = fs::read(file_path).context("failed to read file into memory")?;
    let cursor = Cursor::new(file_bytes);
    Ok(MediaSourceStream::new(Box::new(cursor), Default::default()))
}

pub fn get_media_source_stream(file_path: &str) -> Result<MediaSourceStream> {
    let src = std::fs::File::open(file_path)?;
    Ok(MediaSourceStream::new(Box::new(src), Default::default()))
}

pub fn get_format_reader(file_path: &str) -> Result<Box<dyn FormatReader>> {
    let mss = get_media_source_stream(file_path)?;

    let mut hint = Hint::new();
    if let Some(ext) = file_path.split('.').last() {
        hint.with_extension(ext);
    }

    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();

    let probed = get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

    Ok(probed.format)
}

pub fn stream_audio_file() -> Result<()> {
    AUDIO_MIXER.is_preview_started.store(true, Ordering::SeqCst);
    println!("stream_audio_file");
    let file_path = AUDIO_MIXER
        .preview_file
        .lock()
        .map_err(|_| anyhow::anyhow!("failed to acquire preview file lock"))?
        .clone();
    println!("starting streaming {}", file_path);

    AUDIO_MIXER
        .is_preview_playing
        .store(false, Ordering::SeqCst);

    let mut preview_producer_guard = AUDIO_ENGINE
        .preview_producer
        .lock()
        .map_err(|_| anyhow::anyhow!("Could not lock preview producer"))?;
    let preview_producer = preview_producer_guard
        .as_mut()
        .context("Preview producer missing or stream already started")?;

    let mut format_reader = get_format_reader(file_path.as_str())?;

    let track = format_reader
        .default_track()
        .context("no default track found")?;
    println!("Track sample rate: {:?}", track.codec_params.sample_rate);
    println!("Track channels: {:?}", track.codec_params.channels);

    let track_id = track.id;
    let track_sample_rate = track
        .codec_params
        .sample_rate
        .context("sample rate missing")?;
    let track_channels = track.codec_params.channels.context("channels missing")?;
    let num_track_channels = track_channels.count();

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &dec_opts)
        .context("unsupported codec")?;

    let mut resampler = resampler_2(
        track_sample_rate as usize,
        AUDIO_ENGINE.sample_rate(),
        AUDIO_ENGINE.num_channels(),
    );

    println!(
        "{} {} {}",
        track_sample_rate,
        AUDIO_ENGINE.sample_rate(),
        AUDIO_ENGINE.num_channels()
    );

    let output_delay = resampler.output_delay();
    let mut temp_buffer = SampleBuffer::<f32>::new(
        8192,
        SignalSpec::new(track_sample_rate, track_channels.clone()),
    );
    let mut input_buffer = resampler.input_buffer_allocate(false);
    let mut output_buffer = resampler.output_buffer_allocate(true);
    let mut interleaved_output = [0.0f32; 8192];
    let num_output_frames = 1024;
    let mut removed_delay_frames = false;
    AUDIO_MIXER.is_preview_playing.store(true, Ordering::SeqCst);
    while !AUDIO_MIXER.is_preview_canceled.load(Ordering::SeqCst) {
        if preview_producer.vacant_len() < MINIMUM_BUFFER_SPACE {
            continue;
        }

        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let num_input_frames = decoded.frames();
                temp_buffer.copy_interleaved_ref(decoded);
                let samples = temp_buffer.samples();
                for frame in 0..num_input_frames {
                    for ch in 0..num_track_channels {
                        input_buffer[ch].push(samples[frame * num_track_channels + ch]);
                    }
                    if input_buffer[0].len() == resampler.input_frames_next() {
                        resampler
                            .process_into_buffer(&input_buffer, &mut output_buffer, None)
                            .map_err(|e| anyhow::anyhow!(e))?;
                        for frame in 0..num_output_frames {
                            for ch in 0..num_track_channels {
                                interleaved_output[frame * num_track_channels + ch] =
                                    output_buffer[ch][frame];
                            }
                        }
                        let start_index = if removed_delay_frames {
                            0
                        } else {
                            output_delay * 2
                        };
                        removed_delay_frames = true;
                        for ch in 0..num_track_channels {
                            input_buffer[ch].clear();
                        }
                        preview_producer
                            .push_slice(&interleaved_output[start_index..num_output_frames * 2]);
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if AUDIO_MIXER.is_preview_playing.load(Ordering::SeqCst) {
        for ch in 0..num_track_channels {
            input_buffer[ch].resize(resampler.input_frames_next(), 0.0);
        }
        resampler
            .process_into_buffer(&input_buffer, &mut output_buffer, None)
            .map_err(|e| anyhow::anyhow!(e))?;
        for frame in 0..num_output_frames {
            for ch in 0..num_track_channels {
                interleaved_output[frame * num_track_channels + ch] = output_buffer[ch][frame];
            }
        }
        preview_producer.push_slice(&interleaved_output[..num_output_frames * 2]);
    }
    AUDIO_MIXER
        .is_preview_playing
        .store(false, Ordering::SeqCst);
    AUDIO_MIXER
        .is_preview_started
        .store(false, Ordering::SeqCst);
    Ok(())
}
