// src/audio/decoder.rs

use ringbuf::traits::{Consumer, Observer, Producer};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::sync::{LazyLock, RwLock};
use symphonia::core::audio::{AudioBuffer, SampleBuffer, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use crate::audio::engine::PREVIEW_BUFFER;
use crate::audio::mixer::AUDIO_MIXER;

const MINIMUM_BUFFER_SPACE: usize = 5000;

pub static DECODED_AUDIO_CACHE: LazyLock<RwLock<HashMap<String, Vec<f32>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

fn get_media_source_stream_for_caching(file_path: &str) -> MediaSourceStream {
    let file_bytes = fs::read(file_path).expect("failed to read file into memory");
    let cursor = Cursor::new(file_bytes);
    MediaSourceStream::new(Box::new(cursor), Default::default())
}

fn get_media_source_stream(file_path: &str) -> MediaSourceStream {
    let src = std::fs::File::open(file_path).expect("failed to open media");
    MediaSourceStream::new(Box::new(src), Default::default())
}

fn get_format_reader(file_path: &str) -> Box<dyn FormatReader> {
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

pub fn stream_audio_file(file_path: &str) {
    unsafe {
        AUDIO_MIXER.is_preview_playing = false;
        PREVIEW_BUFFER.clear();
    }

    let mut format_reader = get_format_reader(file_path);

    let track = format_reader
        .default_track()
        .expect("no default track found");
    println!("Track sample rate: {:?}", track.codec_params.sample_rate);
    println!("Track channels: {:?}", track.codec_params.channels);

    let track_id = track.id;

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    let max_samples_per_packet =
        (decoder.codec_params().max_frames_per_packet.unwrap_or(5000) * 2) as usize;
    println!("max_samples_per_packet: {}", max_samples_per_packet);

    unsafe {
        AUDIO_MIXER.is_preview_playing = true;

        while AUDIO_MIXER.is_preview_playing {
            if PREVIEW_BUFFER.vacant_len() < MINIMUM_BUFFER_SPACE {
                std::thread::sleep(std::time::Duration::from_millis(1));
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
                    let mut temp_buffer =
                        SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                    temp_buffer.copy_interleaved_ref(decoded);
                    let samples = temp_buffer.samples();
                    PREVIEW_BUFFER.push_slice(samples);
                }
                Err(_) => continue,
            }
        }
    }
}
