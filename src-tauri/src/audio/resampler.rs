use anyhow::{Context, Result};
use rubato::{
    calculate_cutoff, FastFixedOut, PolynomialDegree, SincFixedOut, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};

use crate::{audio::engine::AUDIO_ENGINE, core::constants::ENGINE_NUM_CHANNELS};

// TODO in the future, extract these options from user preferences

pub fn create_offline_resampler(original_sample_rate: usize) -> Result<SincFixedOut<f32>> {
    let sinc_len = 256;
    let window = WindowFunction::BlackmanHarris2;
    let params = SincInterpolationParameters {
        sinc_len,
        f_cutoff: calculate_cutoff(sinc_len, window),
        interpolation: SincInterpolationType::Cubic,
        oversampling_factor: 256,
        window,
    };
    let resampler = SincFixedOut::<f32>::new(
        AUDIO_ENGINE.sample_rate() as f64 / original_sample_rate as f64,
        2.0,
        params,
        1024,
        AUDIO_ENGINE.num_channels(),
    )
    .context("Failed to create offline resampler")?;

    Ok(resampler)
}

pub fn create_preview_resampler(original_sample_rate: usize) -> Result<FastFixedOut<f32>> {
    let resampler = FastFixedOut::<f32>::new(
        AUDIO_ENGINE.sample_rate() as f64 / original_sample_rate as f64,
        2.0,
        PolynomialDegree::Linear,
        1024,
        AUDIO_ENGINE.num_channels(),
    )
    .context("Failed to create preview resampler")?;

    Ok(resampler)
}
