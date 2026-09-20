use std::io::ErrorKind;
use std::ops::ControlFlow;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use log::warn;
use rubato::Resampler;
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

use crate::core::constants::STEREO_NUM_CHANNELS;
use crate::engine::downmix::DownmixMode;
use crate::engine::resampling_processor::ResamplingProcessor;

/// Reads an audio file and hands it out as interleaved stereo samples at the
/// target sample rate, in chunks. Callers decide what to do with the chunks
/// (fill a ring buffer, collect into memory, ...) and which resampler quality
/// to use.
pub struct AudioFileReader<R: Resampler<f32>> {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    num_source_channels: usize,
    source_sample_rate: u32,
    num_expected_output_samples: Option<usize>,
    downmix_mode: DownmixMode,
    sample_buffer: Option<SampleBuffer<f32>>,
    signal_spec: SignalSpec,
    sample_rate_conversion_mode: SampleRateConversionMode<R>,
}

impl<R: Resampler<f32>> AudioFileReader<R> {
    /// Opens the audio file at `file_path` for reading. Output is stereo at
    /// `target_sample_rate`.
    ///
    /// `make_resampler` builds the resampler used to convert the sample rate.
    /// It receives the file's rate, and is only called when that differs from
    /// `target_sample_rate`.
    pub fn open(
        file_path: &str,
        target_sample_rate: usize,
        make_resampler: impl FnOnce(usize) -> Result<R>,
    ) -> Result<Self> {
        let format_reader = open_format_reader(file_path)?;
        let track = format_reader
            .default_track()
            .context("no default track found")?;
        let track_id = track.id;
        let codec_params = track.codec_params.clone();

        let source_sample_rate = codec_params.sample_rate.context("sample rate missing")?;
        let channels = codec_params.channels.context("channels missing")?;
        let decoder = get_codecs()
            .make(&codec_params, &DecoderOptions::default())
            .context("Unsupported codec")?;

        let num_expected_output_samples = codec_params.n_frames.map(|num_source_frames| {
            let num_target_frames =
                (num_source_frames * target_sample_rate as u64).div_ceil(source_sample_rate as u64);
            num_target_frames as usize * STEREO_NUM_CHANNELS as usize
        });

        let sample_rate_conversion_mode = if source_sample_rate as usize == target_sample_rate {
            SampleRateConversionMode::Passthrough
        } else {
            SampleRateConversionMode::Resample(ResamplingProcessor::new(
                make_resampler(source_sample_rate as usize)?,
                source_sample_rate as u64,
                target_sample_rate as u64,
            ))
        };

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            num_source_channels: channels.count(),
            source_sample_rate,
            num_expected_output_samples,
            downmix_mode: DownmixMode::for_channels(channels),
            sample_buffer: None,
            signal_spec: SignalSpec::new(source_sample_rate, channels),
            sample_rate_conversion_mode,
        })
    }

    pub fn num_source_channels(&self) -> usize {
        self.num_source_channels
    }

    /// Number of samples [`AudioFileReader::for_each_chunk`] will produce in
    /// total, when the source reports its length; useful for sizing a buffer.
    pub fn num_expected_output_samples(&self) -> Option<usize> {
        self.num_expected_output_samples
    }

    pub fn source_sample_rate(&self) -> usize {
        self.source_sample_rate as usize
    }

    /// Runs the whole file through `callback`, one chunk at a time. If `callback`
    /// returns `Break` the read stops immediately and any resampler tail is
    /// discarded; otherwise the tail is flushed once the file ends, so the
    /// output covers exactly the file's length.
    pub fn for_each_chunk(self, mut callback: impl FnMut(&[f32]) -> ControlFlow<()>) -> Result<()> {
        let Self {
            mut format_reader,
            mut decoder,
            track_id,
            num_source_channels,
            downmix_mode,
            mut sample_buffer,
            signal_spec,
            mut sample_rate_conversion_mode,
            ..
        } = self;
        let mut downmixed_stereo_samples: Vec<f32> = Vec::new();

        loop {
            let packet = match format_reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => {
                    // Files with a damaged tail are common; keep what decoded.
                    warn!("Stopped reading audio file early: {e}");
                    break;
                }
            };
            if packet.track_id() != track_id {
                continue;
            }

            let decoded = match decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(e)) => {
                    warn!("Skipping undecodable packet: {e}");
                    continue;
                }
                Err(e) => return Err(anyhow!(e)),
            };

            let num_frames = decoded.frames();
            if num_frames == 0 {
                continue;
            }

            // Capacity is counted in samples (frames * channels), not frames.
            let num_needed_samples = num_frames * num_source_channels;
            let buffer = match &mut sample_buffer {
                Some(buffer) if buffer.capacity() >= num_needed_samples => buffer,
                slot => slot.insert(SampleBuffer::new(num_frames as u64, signal_spec)),
            };
            buffer.copy_interleaved_ref(decoded);

            // Sized to the packet up front and overwritten in place; after the
            // largest packet has been seen this neither allocates nor zero-fills.
            downmixed_stereo_samples.resize(num_frames * STEREO_NUM_CHANNELS as usize, 0.0);
            for (out, frame) in downmixed_stereo_samples
                .chunks_exact_mut(STEREO_NUM_CHANNELS as usize)
                .zip(buffer.samples().chunks_exact(num_source_channels))
            {
                (out[0], out[1]) = downmix_mode.stereo_frame(frame);
            }

            let flow = match &mut sample_rate_conversion_mode {
                SampleRateConversionMode::Passthrough => callback(&downmixed_stereo_samples),
                SampleRateConversionMode::Resample(resampling_processor) => {
                    resampling_processor.push(&downmixed_stereo_samples, &mut callback)?
                }
            };
            if flow.is_break() {
                return Ok(());
            }
        }

        if let SampleRateConversionMode::Resample(resampling_processor) =
            &mut sample_rate_conversion_mode
        {
            resampling_processor.finish(&mut callback)?;
        }
        Ok(())
    }
}

/// What happens to downmixed frames on their way out.
enum SampleRateConversionMode<R: Resampler<f32>> {
    /// The file is already at the target rate: frames go out untouched.
    Passthrough,
    /// The file is at a different rate: frames are resampled to the target
    /// rate.
    Resample(ResamplingProcessor<R>),
}

fn open_format_reader(file_path: &str) -> Result<Box<dyn FormatReader>> {
    let file = std::fs::File::open(file_path).context("Couldn't open file")?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .context("Unknown audio format")?;

    Ok(probed.format)
}

#[cfg(test)]
#[path = "../tests/engine/audio_reader.rs"]
mod tests;
