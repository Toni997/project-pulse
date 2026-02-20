use anyhow::{anyhow, Context, Result};
use log::info;
use ringbuf::traits::{Observer, Producer};
use rubato::Resampler;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::{LazyLock, RwLock};
use std::{f32, fs};
use symphonia::core::audio::{Channels, SampleBuffer, SignalSpec};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use crate::audio::engine::AUDIO_ENGINE;
use crate::audio::preview_mixer::PREVIEW_MIXER;
use crate::audio::resampler::{create_offline_resampler, create_preview_resampler};
use crate::core::constants::ENGINE_NUM_CHANNELS;

const MINIMUM_BUFFER_SPACE: usize = 5000;

#[derive(Default, Clone, Copy)]
struct ChannelIndexMap {
    fl: Option<usize>,
    fr: Option<usize>,
    fc: Option<usize>,
    lfe: Option<usize>,
    sl: Option<usize>,
    sr: Option<usize>,
    rl: Option<usize>,
    rr: Option<usize>,
}

fn build_channel_index_map(channels: Channels) -> ChannelIndexMap {
    let mut map = ChannelIndexMap::default();
    for (idx, ch) in channels.iter().enumerate() {
        match ch {
            Channels::FRONT_LEFT => map.fl = Some(idx),
            Channels::FRONT_RIGHT => map.fr = Some(idx),
            Channels::FRONT_CENTRE => map.fc = Some(idx),
            Channels::LFE1 | Channels::LFE2 => map.lfe = Some(idx),
            Channels::SIDE_LEFT => map.sl = Some(idx),
            Channels::SIDE_RIGHT => map.sr = Some(idx),
            Channels::REAR_LEFT => map.rl = Some(idx),
            Channels::REAR_RIGHT => map.rr = Some(idx),
            _ => {}
        }
    }
    map
}

fn sample_at(frame: &[f32], idx: Option<usize>) -> f32 {
    idx.and_then(|i| frame.get(i)).copied().unwrap_or(0.0)
}

fn downmix_frame_to_stereo(frame: &[f32], map: ChannelIndexMap) -> (f32, f32) {
    // Fast path: if the source is already labeled plain stereo (FL/FR only), don't downmix.
    // This also avoids accidentally swapping channels if FL/FR indices aren't [0, 1].
    if map.fl.is_some()
        && map.fr.is_some()
        && map.fc.is_none()
        && map.lfe.is_none()
        && map.sl.is_none()
        && map.sr.is_none()
        && map.rl.is_none()
        && map.rr.is_none()
    {
        return (sample_at(frame, map.fl), sample_at(frame, map.fr));
    }

    if map.fl.is_some() && map.fr.is_none() && map.fc.is_none() {
        let mono = sample_at(frame, map.fl);
        return (mono, mono);
    }

    if map.fl.is_none() && map.fr.is_none() && map.fc.is_none() {
        let left = *frame.first().unwrap_or(&0.0);
        let right = *frame.get(1).unwrap_or(&left);
        return (left, right);
    }

    let left = sample_at(frame, map.fl)
        + 0.7071 * sample_at(frame, map.fc)
        + 0.7071 * (sample_at(frame, map.sl) + sample_at(frame, map.rl))
        + 0.5 * sample_at(frame, map.lfe);

    let right = sample_at(frame, map.fr)
        + 0.7071 * sample_at(frame, map.fc)
        + 0.7071 * (sample_at(frame, map.sr) + sample_at(frame, map.rr))
        + 0.5 * sample_at(frame, map.lfe);

    (left, right)
}

pub struct DecodedAudioData {
    pub data: Vec<f32>,
    pub original_num_channels: usize,
    pub original_sample_rate: usize,
    pub file_path: String,
    pub file_name: String,
}

pub fn get_media_source_stream_for_caching(file_path: &str) -> Result<MediaSourceStream> {
    let file_bytes = fs::read(file_path).context("Couldn't open file")?;
    let cursor = Cursor::new(file_bytes);
    Ok(MediaSourceStream::new(Box::new(cursor), Default::default()))
}

pub fn get_media_source_stream(file_path: &str) -> Result<MediaSourceStream> {
    let src = std::fs::File::open(file_path).context("Couldn't open file")?;
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

    let probed = get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .context("Unknown audio format")?;

    Ok(probed.format)
}

pub fn stream_audio_file() -> Result<()> {
    PREVIEW_MIXER.is_started.store(true, Ordering::SeqCst);
    info!("stream_audio_file");
    let file_path = PREVIEW_MIXER
        .file_path
        .lock()
        .map_err(|_| anyhow!("failed to acquire preview file lock"))?
        .clone();
    info!("starting streaming {}", file_path);

    PREVIEW_MIXER.is_playing.store(false, Ordering::SeqCst);

    let mut preview_producer_guard = AUDIO_ENGINE
        .preview_producer
        .lock()
        .map_err(|_| anyhow!("Could not lock preview producer"))?;
    let preview_producer = preview_producer_guard
        .as_mut()
        .context("Preview producer missing or stream already started")?;

    let mut format_reader = get_format_reader(file_path.as_str())?;

    let track = format_reader
        .default_track()
        .context("no default track found")?;
    info!("Track sample rate: {:?}", track.codec_params.sample_rate);
    info!("Track channels: {:?}", track.codec_params.channels);

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
        .context("Unsupported codec")?;

    let mut resampler = create_preview_resampler(track_sample_rate as usize)?;
    let engine_channels = AUDIO_ENGINE.num_channels();
    let channel_map = build_channel_index_map(track_channels);

    info!(
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
    let mut interleaved_output = vec![0.0f32; 8192];
    let mut removed_delay_frames = false;
    PREVIEW_MIXER.is_playing.store(true, Ordering::SeqCst);
    while !PREVIEW_MIXER.is_canceled.load(Ordering::SeqCst) {
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
                    let frame_start = frame * num_track_channels;
                    let frame_end = frame_start + num_track_channels;
                    let frame_samples = &samples[frame_start..frame_end];

                    let (left, right) = downmix_frame_to_stereo(frame_samples, channel_map);
                    input_buffer[0].push(left);
                    input_buffer[1].push(right);
                    if input_buffer[0].len() == resampler.input_frames_next() {
                        resampler
                            .process_into_buffer(&input_buffer, &mut output_buffer, None)
                            .map_err(|e| anyhow!(e))?;
                        let out_frames = output_buffer[0].len();
                        let needed = out_frames * engine_channels;
                        if interleaved_output.len() < needed {
                            interleaved_output.resize(needed, 0.0);
                        }
                        for i in 0..out_frames {
                            for ch in 0..engine_channels {
                                interleaved_output[i * engine_channels + ch] = output_buffer[ch][i];
                            }
                        }
                        let start_index = if removed_delay_frames {
                            0
                        } else {
                            output_delay * engine_channels
                        };
                        removed_delay_frames = true;
                        for ch in 0..engine_channels {
                            input_buffer[ch].clear();
                        }
                        if start_index < needed {
                            preview_producer.push_slice(&interleaved_output[start_index..needed]);
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }
    if PREVIEW_MIXER.is_playing.load(Ordering::SeqCst) {
        for ch in 0..engine_channels {
            input_buffer[ch].resize(resampler.input_frames_next(), 0.0);
        }
        resampler
            .process_into_buffer(&input_buffer, &mut output_buffer, None)
            .map_err(|e| anyhow!(e))?;
        let out_frames = output_buffer[0].len();
        let needed = out_frames * engine_channels;
        if interleaved_output.len() < needed {
            interleaved_output.resize(needed, 0.0);
        }
        for i in 0..out_frames {
            for ch in 0..engine_channels {
                interleaved_output[i * engine_channels + ch] = output_buffer[ch][i];
            }
        }
        preview_producer.push_slice(&interleaved_output[..needed]);
    }
    PREVIEW_MIXER.is_playing.store(false, Ordering::SeqCst);
    PREVIEW_MIXER.is_started.store(false, Ordering::SeqCst);
    Ok(())
}

pub fn decode_audio_file(file_path: String) -> Result<DecodedAudioData> {
    let mut format_reader = get_format_reader(&file_path)?;

    let track = format_reader
        .default_track()
        .context("no default track found")?;

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
        .context("Unsupported codec")?;

    let engine_channels = AUDIO_ENGINE.num_channels();

    let mut resampler = create_preview_resampler(track_sample_rate as usize)?;
    let channel_map = build_channel_index_map(track_channels);

    let mut decoded_data: Vec<f32> = Vec::new();
    let mut input_buffer = resampler.input_buffer_allocate(false);
    let mut output_buffer = resampler.output_buffer_allocate(true);

    let mut temp_buffer = SampleBuffer::<f32>::new(
        track.codec_params.max_frames_per_packet.unwrap_or(4096) as u64,
        SignalSpec::new(track_sample_rate, track_channels.clone()),
    );

    loop {
        let packet = match format_reader.next_packet() {
            Ok(packet) => packet,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let num_frames = decoded.frames();
                if num_frames == 0 {
                    continue;
                }

                if temp_buffer.capacity() < num_frames {
                    temp_buffer = SampleBuffer::<f32>::new(
                        num_frames as u64,
                        SignalSpec::new(track_sample_rate, track_channels.clone()),
                    );
                }

                temp_buffer.copy_interleaved_ref(decoded);
                let samples = temp_buffer.samples();

                for frame in 0..num_frames {
                    let frame_start = frame * num_track_channels;
                    let frame_end = frame_start + num_track_channels;
                    let frame_samples = &samples[frame_start..frame_end];

                    let (left, right) = downmix_frame_to_stereo(frame_samples, channel_map);
                    input_buffer[0].push(left);
                    input_buffer[1].push(right);

                    if input_buffer[0].len() >= resampler.input_frames_next() {
                        resampler
                            .process_into_buffer(&input_buffer, &mut output_buffer, None)
                            .map_err(|e| anyhow!(e))?;

                        let out_frames = output_buffer[0].len();
                        for i in 0..out_frames {
                            for ch in 0..engine_channels {
                                decoded_data.push(output_buffer[ch][i]);
                            }
                        }

                        for ch in 0..engine_channels {
                            input_buffer[ch].clear();
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    // Flush remaining samples
    let needed = resampler.input_frames_next();
    let current = input_buffer[0].len();
    if current > 0 {
        for ch in 0..engine_channels {
            input_buffer[ch].resize(needed, 0.0);
        }
        resampler
            .process_into_buffer(&input_buffer, &mut output_buffer, None)
            .map_err(|e| anyhow!(e))?;

        let num_out_frames = output_buffer[0].len();
        for i in 0..num_out_frames {
            for ch in 0..engine_channels {
                decoded_data.push(output_buffer[ch][i]);
            }
        }
    }

    // Remove delay
    let output_delay = resampler.output_delay();
    let samples_to_remove = output_delay * engine_channels;
    if decoded_data.len() >= samples_to_remove {
        decoded_data.drain(0..samples_to_remove);
    }

    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(DecodedAudioData {
        data: decoded_data,
        original_num_channels: num_track_channels,
        original_sample_rate: track_sample_rate as usize,
        file_path,
        file_name,
    })
}
