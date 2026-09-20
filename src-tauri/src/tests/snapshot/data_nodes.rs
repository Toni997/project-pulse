use std::sync::mpsc;

use super::*;
use crate::engine::decoder::DecodedAudioData;
use crate::project::clip::NewClip;

struct Fixture {
    nodes: DataNodes,
    track_id: Id,
    clip_id: Id,
}

fn decoded_audio() -> DecodedAudioData {
    DecodedAudioData {
        data: vec![0.0; 100],
        original_num_channels: 2,
        original_sample_rate: 48000,
        file_path: "a.wav".to_string(),
        file_name: "a.wav".to_string(),
    }
}

/// A project with one audio track holding one clip whose audio is `source_id`
/// (or, if `None`, the audio that was actually loaded into the pool).
fn build(source_id: Option<Id>) -> Fixture {
    let (changes, _receiver) = mpsc::channel();
    let project_state = ProjectState::new(changes);
    let asset_pool = AssetPool::new();
    let loaded_id = asset_pool.audio.add(decoded_audio());

    let track = project_state.add_audio_track_with_clip(NewClip {
        name: "clip".to_string(),
        start_ppq: 0,
        length_ppq: 10,
        source_id: source_id.unwrap_or(loaded_id),
    });
    let clip_id = track.clips.keys().next().unwrap().clone();

    Fixture {
        nodes: DataNodes::build(&project_state, &asset_pool, || false).unwrap(),
        track_id: track.id.clone(),
        clip_id,
    }
}

#[test]
fn a_node_is_found_as_its_own_type() {
    let fixture = build(None);
    assert!(fixture
        .nodes
        .get_node::<ClipNode>(&fixture.clip_id)
        .is_some());
    assert!(fixture
        .nodes
        .get_node::<TrackNode>(&fixture.track_id)
        .is_some());
    assert!(fixture.nodes.get_node::<MasterNode>("master").is_some());
}

#[test]
fn a_node_is_not_found_as_a_different_type() {
    let fixture = build(None);
    assert!(fixture
        .nodes
        .get_node::<TrackNode>(&fixture.clip_id)
        .is_none());
    assert!(fixture
        .nodes
        .get_node::<ClipNode>(&fixture.track_id)
        .is_none());
}

#[test]
fn an_unknown_id_finds_nothing() {
    let fixture = build(None);
    assert!(fixture.nodes.get_node::<ClipNode>("nope").is_none());
}

#[test]
fn a_clip_whose_audio_is_not_loaded_gets_no_node() {
    let fixture = build(Some("not-in-the-pool".to_string()));
    assert!(fixture
        .nodes
        .get_node::<ClipNode>(&fixture.clip_id)
        .is_none());
    assert!(fixture
        .nodes
        .get_node::<TrackNode>(&fixture.track_id)
        .is_some());
}

#[test]
fn a_build_stops_when_an_edit_lands_while_it_runs() {
    let (changes, _requests) = mpsc::channel();
    let project_state = ProjectState::new(changes);
    let asset_pool = AssetPool::new();
    project_state.add_audio_track();
    project_state.add_audio_track();

    let version_at_start = project_state.version();
    let checks = std::cell::Cell::new(0);
    let result = DataNodes::build(&project_state, &asset_pool, || {
        checks.set(checks.get() + 1);
        if checks.get() == 2 {
            // An edit arrives part way through the build.
            project_state.add_audio_track();
        }
        project_state.version() != version_at_start
    });

    assert!(result.is_none());
}
