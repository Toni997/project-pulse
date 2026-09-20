use symphonia::core::audio::Channels;

// Gains applied when folding surround channels into left/right. Centre and
// side/rear channels are attenuated by 3 dB (1/sqrt(2)) so that a signal spread
// across two speakers keeps roughly the same overall power; the low-frequency
// effects channel is attenuated by 6 dB.
const CENTRE_GAIN: f32 = std::f32::consts::FRAC_1_SQRT_2;
const SIDE_REAR_GAIN: f32 = std::f32::consts::FRAC_1_SQRT_2;
const LFE_GAIN: f32 = 0.5;

/// How a channel layout is folded down to two channels (left and right).
///
/// Chosen from a layout with [`DownmixMode::for_channels`]; converts one frame
/// of samples with [`DownmixMode::stereo_frame`].
#[derive(Clone, Copy)]
pub enum DownmixMode {
    /// Plain labeled stereo (front left/right only): no mixing needed. Using
    /// the labels also avoids swapping channels if they aren't at [0, 1].
    Stereo { left: usize, right: usize },
    /// A lone front-left channel, duplicated to both sides.
    Mono { index: usize },
    /// Channels without usable labels: take the first two as left/right.
    FirstTwo,
    /// Anything else (centre, LFE, sides, rears): fold into left/right.
    Surround(SurroundMap),
}

/// Where each speaker position sits in a source's interleaved frame.
#[derive(Default, Clone, Copy)]
pub struct SurroundMap {
    fl: Option<usize>,
    fr: Option<usize>,
    fc: Option<usize>,
    lfe: Option<usize>,
    sl: Option<usize>,
    sr: Option<usize>,
    rl: Option<usize>,
    rr: Option<usize>,
}

impl DownmixMode {
    pub fn for_channels(channels: Channels) -> Self {
        let mut map = SurroundMap::default();
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

        match map {
            SurroundMap {
                fl: Some(left),
                fr: Some(right),
                fc: None,
                lfe: None,
                sl: None,
                sr: None,
                rl: None,
                rr: None,
            } => DownmixMode::Stereo { left, right },
            SurroundMap {
                fl: Some(index),
                fr: None,
                fc: None,
                ..
            } => DownmixMode::Mono { index },
            SurroundMap {
                fl: None,
                fr: None,
                fc: None,
                ..
            } => DownmixMode::FirstTwo,
            _ => DownmixMode::Surround(map),
        }
    }

    pub fn stereo_frame(&self, frame: &[f32]) -> (f32, f32) {
        match *self {
            DownmixMode::Stereo { left, right } => {
                (sample_at(frame, Some(left)), sample_at(frame, Some(right)))
            }
            DownmixMode::Mono { index } => {
                let mono = sample_at(frame, Some(index));
                (mono, mono)
            }
            DownmixMode::FirstTwo => {
                let left = *frame.first().unwrap_or(&0.0);
                let right = *frame.get(1).unwrap_or(&left);
                (left, right)
            }
            DownmixMode::Surround(map) => {
                let left = sample_at(frame, map.fl)
                    + CENTRE_GAIN * sample_at(frame, map.fc)
                    + SIDE_REAR_GAIN * (sample_at(frame, map.sl) + sample_at(frame, map.rl))
                    + LFE_GAIN * sample_at(frame, map.lfe);

                let right = sample_at(frame, map.fr)
                    + CENTRE_GAIN * sample_at(frame, map.fc)
                    + SIDE_REAR_GAIN * (sample_at(frame, map.sr) + sample_at(frame, map.rr))
                    + LFE_GAIN * sample_at(frame, map.lfe);

                (left, right)
            }
        }
    }
}

fn sample_at(frame: &[f32], idx: Option<usize>) -> f32 {
    idx.and_then(|i| frame.get(i)).copied().unwrap_or(0.0)
}

#[cfg(test)]
#[path = "../tests/engine/downmix.rs"]
mod tests;
