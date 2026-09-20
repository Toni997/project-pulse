use super::*;
use rubato::{
    calculate_cutoff, FastFixedOut, PolynomialDegree, SincFixedOut, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};

fn sinc(ratio: f64) -> SincFixedOut<f32> {
    let window = WindowFunction::BlackmanHarris2;
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: calculate_cutoff(256, window),
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window,
    };
    SincFixedOut::<f32>::new(ratio, 2.0, params, 1024, 2).unwrap()
}

fn fast(ratio: f64) -> FastFixedOut<f32> {
    FastFixedOut::<f32>::new(ratio, 2.0, PolynomialDegree::Linear, 1024, 2).unwrap()
}

/// Feeds `frames` constant stereo frames through a `ResamplingProcessor` in pieces of
/// `piece_frames` and collects everything it emits.
fn run<R: Resampler<f32>>(
    resampler: R,
    source_rate: u64,
    target_rate: u64,
    frames: u64,
    piece_frames: u64,
) -> Vec<f32> {
    let mut processor = ResamplingProcessor::new(resampler, source_rate, target_rate);
    let mut out = Vec::new();
    let mut callback = |chunk: &[f32]| {
        out.extend_from_slice(chunk);
        ControlFlow::Continue(())
    };
    let mut remaining = frames;
    while remaining > 0 {
        let piece = piece_frames.min(remaining);
        let stereo = vec![0.5f32; piece as usize * STEREO_NUM_CHANNELS as usize];
        let _ = processor.push(&stereo, &mut callback).unwrap();
        remaining -= piece;
    }
    processor.finish(&mut callback).unwrap();
    out
}

fn expected_frames(frames: u64, source_rate: u64, target_rate: u64) -> u64 {
    (frames * target_rate).div_ceil(source_rate)
}

fn assert_length_and_level(out: &[f32], source_rate: u64, target_rate: u64, frames: u64) {
    assert_eq!(
        out.len() as u64,
        expected_frames(frames, source_rate, target_rate) * 2
    );
    let mid = out.len() / 2;
    assert!((out[mid] - 0.5).abs() < 0.01, "mid sample was {}", out[mid]);
}

#[test]
fn sinc_upsample_output_length_matches_input_duration() {
    let out = run(sinc(48000.0 / 44100.0), 44100, 48000, 44100, 44100);
    assert_length_and_level(&out, 44100, 48000, 44100);
}

#[test]
fn sinc_downsample_output_length_matches_input_duration() {
    let out = run(sinc(44100.0 / 96000.0), 96000, 44100, 50000, 50000);
    assert_length_and_level(&out, 96000, 44100, 50000);
}

#[test]
fn fast_resampler_output_length_matches_input_duration() {
    let out = run(fast(48000.0 / 44100.0), 44100, 48000, 30000, 30000);
    assert_length_and_level(&out, 44100, 48000, 30000);
}

#[test]
fn input_shorter_than_one_chunk_still_produces_full_length() {
    let out = run(sinc(48000.0 / 44100.0), 44100, 48000, 300, 300);
    assert_eq!(out.len() as u64, expected_frames(300, 44100, 48000) * 2);
}

#[test]
fn empty_input_produces_no_output() {
    let out = run(sinc(48000.0 / 44100.0), 44100, 48000, 0, 1);
    assert!(out.is_empty());
}

#[test]
fn output_does_not_depend_on_how_input_is_split_into_pieces() {
    let whole = run(sinc(48000.0 / 44100.0), 44100, 48000, 20000, 20000);
    let pieces = run(sinc(48000.0 / 44100.0), 44100, 48000, 20000, 777);
    assert_eq!(whole, pieces);
}
