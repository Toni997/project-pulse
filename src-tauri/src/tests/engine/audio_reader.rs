use super::*;
use rubato::{FastFixedOut, PolynomialDegree};

fn fast(ratio: f64) -> FastFixedOut<f32> {
    FastFixedOut::<f32>::new(ratio, 2.0, PolynomialDegree::Linear, 1024, 2).unwrap()
}

fn expected_frames(frames: u64, source_rate: u64, target_rate: u64) -> u64 {
    (frames * target_rate).div_ceil(source_rate)
}

fn write_wav(name: &str, sample_rate: u32, frames: &[(i16, i16)]) -> std::path::PathBuf {
    let data_len = (frames.len() * 4) as u32;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&2u16.to_le_bytes()); // channels
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate * 4).to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes()); // block align
    bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for (left, right) in frames {
        bytes.extend_from_slice(&left.to_le_bytes());
        bytes.extend_from_slice(&right.to_le_bytes());
    }
    let path = std::env::temp_dir().join(format!("pulse_audio_reader_{name}.wav"));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn read_all<R: Resampler<f32>>(
    path: &std::path::Path,
    target_rate: usize,
    make_resampler: impl FnOnce(usize) -> Result<R>,
) -> Vec<f32> {
    let reader =
        AudioFileReader::open(path.to_str().unwrap(), target_rate, make_resampler).unwrap();
    let mut out = Vec::new();
    reader
        .for_each_chunk(|chunk| {
            out.extend_from_slice(chunk);
            ControlFlow::Continue(())
        })
        .unwrap();
    out
}

fn ramp(frames: usize) -> Vec<(i16, i16)> {
    (0..frames)
        .map(|i| {
            (
                (i as i32 % 20000 - 10000) as i16,
                (10000 - i as i32 % 20000) as i16,
            )
        })
        .collect()
}

#[test]
fn file_at_target_rate_passes_through_unchanged_and_never_builds_a_resampler() {
    let frames = ramp(10_000);
    let path = write_wav("passthrough", 48000, &frames);

    let out = read_all(&path, 48000, |_| -> Result<FastFixedOut<f32>> {
        panic!("no resampler should be built when rates match")
    });
    std::fs::remove_file(&path).ok();

    assert_eq!(out.len(), frames.len() * 2);
    for (i, (left, right)) in frames.iter().enumerate() {
        assert!((out[i * 2] - *left as f32 / 32768.0).abs() < 1e-6);
        assert!((out[i * 2 + 1] - *right as f32 / 32768.0).abs() < 1e-6);
    }
}

#[test]
fn file_at_other_rate_is_resampled_to_the_expected_length() {
    let frames = ramp(30_000);
    let path = write_wav("resampled", 44100, &frames);

    let out = read_all(&path, 48000, |source_rate| {
        Ok(fast(48000.0 / source_rate as f64))
    });
    std::fs::remove_file(&path).ok();

    assert_eq!(
        out.len() as u64,
        expected_frames(frames.len() as u64, 44100, 48000) * 2
    );
}

#[test]
fn breaking_from_the_callback_stops_reading() {
    let path = write_wav("break", 48000, &ramp(50_000));
    let reader = AudioFileReader::open(
        path.to_str().unwrap(),
        48000,
        |_| -> Result<FastFixedOut<f32>> { unreachable!() },
    )
    .unwrap();

    let mut calls = 0;
    reader
        .for_each_chunk(|_| {
            calls += 1;
            ControlFlow::Break(())
        })
        .unwrap();
    std::fs::remove_file(&path).ok();

    assert_eq!(calls, 1);
}

#[test]
fn expected_output_samples_matches_what_is_actually_produced() {
    for (rate, name) in [
        (48000u32, "expected_same_rate"),
        (44100, "expected_resampled"),
    ] {
        let path = write_wav(name, rate, &ramp(30_000));
        let reader = AudioFileReader::open(path.to_str().unwrap(), 48000, |source_rate| {
            Ok(fast(48000.0 / source_rate as f64))
        })
        .unwrap();

        let expected = reader.num_expected_output_samples();
        let mut produced = 0;
        reader
            .for_each_chunk(|chunk| {
                produced += chunk.len();
                ControlFlow::Continue(())
            })
            .unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(expected, Some(produced), "source rate {rate}");
    }
}
