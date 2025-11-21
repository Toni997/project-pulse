use rubato::{
    calculate_cutoff, FastFixedIn, FftFixedIn, PolynomialDegree, Resampler, SincFixedIn,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

// TODO in the future, extract these options from user preferences

pub fn resampler(
    original_sample_rate: usize,
    wanted_sample_rate: usize,
    num_channels: usize,
) -> SincFixedIn<f32> {
    let params = SincInterpolationParameters {
        sinc_len: 16,
        f_cutoff: calculate_cutoff(16, WindowFunction::Hann),
        interpolation: SincInterpolationType::Cubic, // use cubic or quadratic for offline
        oversampling_factor: 32,                     // use 256 for offline
        window: WindowFunction::Hann,                // use BlackmanHarris2 for offline
    };
    let resampler = SincFixedIn::<f32>::new(
        wanted_sample_rate as f64 / original_sample_rate as f64,
        2.0,
        params,
        1024,
        num_channels,
    )
    .unwrap();

    resampler
}

pub fn resampler_2(
    original_sample_rate: usize,
    wanted_sample_rate: usize,
    num_channels: usize,
) -> FastFixedIn<f32> {
    let resampler = FastFixedIn::<f32>::new(
        wanted_sample_rate as f64 / original_sample_rate as f64,
        2.0,
        PolynomialDegree::Linear,
        1024,
        num_channels,
    )
    .unwrap();

    resampler
}
