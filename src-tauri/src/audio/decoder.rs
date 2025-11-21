// src/audio/decoder.rs

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
use symphonia::core::probe::Hint;
use symphonia::core::sample;
use symphonia::default::{get_codecs, get_probe};

use crate::audio::engine::{AUDIO_ENGINE, PREVIEW_BUFFER};
use crate::audio::mixer::{
    AUDIO_MIXER, PREVIEW_ALREADY_QUEUED, PREVIEW_AUDIO_PATH, PREVIEW_AUDIO_RUNNING,
    PREVIEW_CANCEL_FLAG,
};
use crate::audio::resampler::{resampler, resampler_2};

const MINIMUM_BUFFER_SPACE: usize = 5000;

pub static DECODED_AUDIO_CACHE: LazyLock<RwLock<HashMap<String, Vec<f32>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn get_media_source_stream_for_caching(file_path: &str) -> MediaSourceStream {
    let file_bytes = fs::read(file_path).expect("failed to read file into memory");
    let cursor = Cursor::new(file_bytes);
    MediaSourceStream::new(Box::new(cursor), Default::default())
}

pub fn get_media_source_stream(file_path: &str) -> MediaSourceStream {
    let src = std::fs::File::open(file_path).expect("failed to open media");
    MediaSourceStream::new(Box::new(src), Default::default())
}

pub fn get_format_reader(file_path: &str) -> Box<dyn FormatReader> {
    let mss = get_media_source_stream(file_path);

    let mut hint = Hint::new();
    if let Some(ext) = file_path.split('.').last() {
        hint.with_extension(ext);
    }

    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();

    let probed = get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");

    return probed.format;
}

pub fn stream_audio_file() {
    PREVIEW_AUDIO_RUNNING.store(true, Ordering::SeqCst);
    let file_path = PREVIEW_AUDIO_PATH.lock().unwrap();
    println!("starting streaming {}", file_path);
    unsafe {
        AUDIO_MIXER
            .is_preview_playing
            .store(false, Ordering::SeqCst);
        PREVIEW_BUFFER.clear();
    }

    let mut format_reader = get_format_reader(file_path.as_str());

    let track = format_reader
        .default_track()
        .expect("no default track found");
    println!("Track sample rate: {:?}", track.codec_params.sample_rate);
    println!("Track channels: {:?}", track.codec_params.channels);

    let track_id = track.id;
    let track_sample_rate = track.codec_params.sample_rate.unwrap();
    let track_channels = track.codec_params.channels.unwrap();
    let num_track_channels = track_channels.count();

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    unsafe {
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
        let num_output_frames = 1024 * AUDIO_ENGINE.sample_rate() / track_sample_rate as usize;
        let mut buffer_filled_once = false;
        let mut removed_delay_frames = false;
        while !PREVIEW_CANCEL_FLAG.load(Ordering::SeqCst) {
            if PREVIEW_BUFFER.vacant_len() < MINIMUM_BUFFER_SPACE {
                // std::thread::sleep(std::time::Duration::from_millis(1697));
                if !buffer_filled_once {
                    AUDIO_MIXER.is_preview_playing.store(true, Ordering::SeqCst);
                    buffer_filled_once = true;
                }
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
                    // PREVIEW_BUFFER.push_slice(&samples); // for resampled version remove this line and uncomment the rest below
                    for frame in 0..num_input_frames {
                        for ch in 0..num_track_channels {
                            input_buffer[ch].push(samples[frame * num_track_channels + ch]);
                        }
                        if input_buffer[0].len() == 1024 {
                            resampler
                                .process_into_buffer(&input_buffer, &mut output_buffer, None)
                                .unwrap();
                            for frame in 0..num_output_frames {
                                for ch in 0..num_track_channels {
                                    interleaved_output[frame * num_track_channels + ch] =
                                        output_buffer[ch][frame];
                                }
                            }
                            let start_index = if removed_delay_frames {
                                0
                            } else {
                                output_delay
                            };
                            if !removed_delay_frames {
                                // let max = interleaved_output[start_index];
                                let max = 0.5f32;
                                let min = 0.0f32;
                                let n = 1000;
                                let step = (max - min) / (n - 1) as f32;
                                println!("max: {}, min: {}, step: {}", max, min, step);

                                let values: [f32; 1000] =
                                    std::array::from_fn(|i| min + i as f32 * step);
                                // for v in 0..1000 {
                                //     print!("{},", values[v]);
                                // }
                                // PREVIEW_BUFFER.push_slice(&values);
                            }
                            removed_delay_frames = true;
                            for ch in 0..num_track_channels {
                                input_buffer[ch].clear();
                            }
                            PREVIEW_BUFFER.push_slice(
                                &interleaved_output[start_index..num_output_frames * 2],
                            );
                        }
                    }
                }
                Err(_) => continue,
            }
        }
        if AUDIO_MIXER.is_preview_playing.load(Ordering::SeqCst) {
            for ch in 0..num_track_channels {
                input_buffer[ch].resize(1024, 0.0);
            }
            resampler
                .process_into_buffer(&input_buffer, &mut output_buffer, None)
                .unwrap();
            for frame in 0..num_output_frames {
                for ch in 0..num_track_channels {
                    interleaved_output[frame * num_track_channels + ch] = output_buffer[ch][frame];
                }
            }
            PREVIEW_BUFFER.push_slice(&interleaved_output[..num_output_frames * 2]);
        }
        PREVIEW_AUDIO_RUNNING.store(false, Ordering::SeqCst)
    }
}
