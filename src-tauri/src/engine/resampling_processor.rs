use std::ops::ControlFlow;

use anyhow::{anyhow, Result};
use rubato::Resampler;

use crate::core::constants::STEREO_NUM_CHANNELS;

/// Feeds stereo frames into a fixed-output-size resampler and emits the
/// result as interleaved chunks, compensating for the resampler's start-up
/// delay and trimming the end so the output length matches the input.
pub struct ResamplingProcessor<R: Resampler<f32>> {
    resampler: R,
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>,
    interleaved: Vec<f32>,
    num_delay_frames_left: usize,
    num_input_frames: u64,
    num_emitted_frames: u64,
    source_rate: u64,
    target_rate: u64,
}

impl<R: Resampler<f32>> ResamplingProcessor<R> {
    pub fn new(resampler: R, source_rate: u64, target_rate: u64) -> Self {
        Self {
            input: resampler.input_buffer_allocate(false),
            output: resampler.output_buffer_allocate(true),
            num_delay_frames_left: resampler.output_delay(),
            resampler,
            interleaved: Vec::new(),
            num_input_frames: 0,
            num_emitted_frames: 0,
            source_rate,
            target_rate,
        }
    }

    /// Accepts any number of interleaved stereo frames.
    pub fn push(
        &mut self,
        stereo: &[f32],
        callback: &mut impl FnMut(&[f32]) -> ControlFlow<()>,
    ) -> Result<ControlFlow<()>> {
        for frame in stereo.chunks_exact(STEREO_NUM_CHANNELS as usize) {
            self.input[0].push(frame[0]);
            self.input[1].push(frame[1]);
            self.num_input_frames += 1;

            if self.input[0].len() == self.resampler.input_frames_next()
                && self.process(u64::MAX, callback)?.is_break()
            {
                return Ok(ControlFlow::Break(()));
            }
        }
        Ok(ControlFlow::Continue(()))
    }

    /// Pushes the buffered tail (zero-padded) through the resampler until
    /// every real input frame has come out the other end, then stops exactly
    /// at the expected output length.
    pub fn finish(&mut self, callback: &mut impl FnMut(&[f32]) -> ControlFlow<()>) -> Result<()> {
        let num_expected_frames =
            (self.num_input_frames * self.target_rate).div_ceil(self.source_rate.max(1));

        while self.num_emitted_frames < num_expected_frames {
            let num_needed = self.resampler.input_frames_next();
            for channel in &mut self.input {
                channel.resize(num_needed, 0.0);
            }
            if self.process(num_expected_frames, callback)?.is_break() {
                break;
            }
        }
        Ok(())
    }

    /// Resamples the buffered input and emits at most `limit - emitted`
    /// frames (after skipping the start-up delay).
    fn process(
        &mut self,
        limit: u64,
        callback: &mut impl FnMut(&[f32]) -> ControlFlow<()>,
    ) -> Result<ControlFlow<()>> {
        let (_, num_out_frames) = self
            .resampler
            .process_into_buffer(&self.input, &mut self.output, None)
            .map_err(|e| anyhow!(e))?;
        for channel in &mut self.input {
            channel.clear();
        }

        let num_skipped = self.num_delay_frames_left.min(num_out_frames);
        self.num_delay_frames_left -= num_skipped;
        let num_available = (num_out_frames - num_skipped) as u64;
        let num_to_emit = num_available.min(limit.saturating_sub(self.num_emitted_frames)) as usize;
        if num_to_emit == 0 {
            return Ok(ControlFlow::Continue(()));
        }

        self.interleaved.clear();
        for i in num_skipped..num_skipped + num_to_emit {
            for ch in 0..STEREO_NUM_CHANNELS as usize {
                self.interleaved.push(self.output[ch][i]);
            }
        }
        self.num_emitted_frames += num_to_emit as u64;
        Ok(callback(&self.interleaved))
    }
}

#[cfg(test)]
#[path = "../tests/engine/resampling_processor.rs"]
mod tests;
