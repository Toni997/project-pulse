use std::ops::ControlFlow;
use std::path::Path;

use anyhow::Result;

use crate::engine::audio_reader::AudioFileReader;
use crate::engine::device::AudioEngine;
use crate::engine::resampler_builder::create_offline_resampler;

pub struct DecodedAudioData {
    pub data: Vec<f32>,
    pub original_num_channels: usize,
    pub original_sample_rate: usize,
    pub file_path: String,
    pub file_name: String,
}

/// Decodes a whole file into memory as engine-format samples, using the
/// high-quality offline resampler since this is what ends up in the project.
pub fn decode_audio_file(
    file_path: String,
    audio_engine: &AudioEngine,
) -> Result<DecodedAudioData> {
    let reader = AudioFileReader::open(&file_path, audio_engine.sample_rate(), |source_rate| {
        create_offline_resampler(source_rate, audio_engine)
    })?;
    let original_num_channels = reader.num_source_channels();
    let original_sample_rate = reader.source_sample_rate();

    let mut data = Vec::with_capacity(reader.num_expected_output_samples().unwrap_or(0));
    reader.for_each_chunk(|chunk| {
        data.extend_from_slice(chunk);
        ControlFlow::Continue(())
    })?;

    let file_name = Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(DecodedAudioData {
        data,
        original_num_channels,
        original_sample_rate,
        file_path,
        file_name,
    })
}
