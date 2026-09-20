use super::*;

#[test]
fn labeled_stereo_passes_through() {
    let downmix_mode = DownmixMode::for_channels(Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
    assert!(matches!(
        downmix_mode,
        DownmixMode::Stereo { left: 0, right: 1 }
    ));
    assert_eq!(downmix_mode.stereo_frame(&[0.25, -0.5]), (0.25, -0.5));
}

#[test]
fn front_left_only_is_duplicated_to_both_sides() {
    let downmix_mode = DownmixMode::for_channels(Channels::FRONT_LEFT);
    assert!(matches!(downmix_mode, DownmixMode::Mono { index: 0 }));
    assert_eq!(downmix_mode.stereo_frame(&[0.5]), (0.5, 0.5));
}

#[test]
fn unlabeled_channels_use_first_two_samples() {
    let downmix_mode = DownmixMode::for_channels(Channels::empty());
    assert!(matches!(downmix_mode, DownmixMode::FirstTwo));
    assert_eq!(downmix_mode.stereo_frame(&[0.1, 0.2, 0.3]), (0.1, 0.2));
    assert_eq!(downmix_mode.stereo_frame(&[0.4]), (0.4, 0.4));
}

#[test]
fn surround_folds_centre_into_both_sides() {
    let downmix_mode = DownmixMode::for_channels(
        Channels::FRONT_LEFT | Channels::FRONT_RIGHT | Channels::FRONT_CENTRE,
    );
    assert!(matches!(downmix_mode, DownmixMode::Surround(_)));
    let (l, r) = downmix_mode.stereo_frame(&[0.0, 0.0, 1.0]);
    assert!(
        (l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6
            && (r - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6
    );
}

#[test]
fn stereo_with_an_lfe_channel_is_not_treated_as_plain_stereo() {
    let downmix_mode =
        DownmixMode::for_channels(Channels::FRONT_LEFT | Channels::FRONT_RIGHT | Channels::LFE1);
    assert!(matches!(downmix_mode, DownmixMode::Surround(_)));
    let (l, r) = downmix_mode.stereo_frame(&[0.2, 0.4, 1.0]);
    assert!((l - 0.7).abs() < 1e-6 && (r - 0.9).abs() < 1e-6);
}

#[test]
fn short_frames_read_missing_channels_as_silence() {
    let downmix_mode = DownmixMode::for_channels(Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
    assert_eq!(downmix_mode.stereo_frame(&[0.3]), (0.3, 0.0));
}
