use std::sync::mpsc;

use super::*;
use crate::engine::decoder::DecodedAudioData;
use crate::project::clip::NewClip;

/// Builds `DataNodes` holding one clip node per entry of `clip_audio` (each
/// entry is that clip's audio), and returns the clip ids in the same order.
fn data_nodes_with_clips(clip_audio: Vec<Vec<f32>>) -> (DataNodes, Vec<Id>) {
    let (changes, _receiver) = mpsc::channel();
    let project_state = ProjectState::new(changes);
    let asset_pool = AssetPool::new();

    let new_clip = |index: usize, audio: Vec<f32>| {
        let source_id = asset_pool.audio.add(DecodedAudioData {
            data: audio,
            original_num_channels: 2,
            original_sample_rate: 48000,
            file_path: format!("clip_{index}.wav"),
            file_name: format!("clip_{index}.wav"),
        });
        NewClip {
            name: format!("clip {index}"),
            start_ppq: 0,
            length_ppq: 0,
            source_id,
        }
    };

    let mut audio = clip_audio.into_iter().enumerate();
    let (first_index, first_audio) = audio.next().unwrap();
    let track = project_state.add_audio_track_with_clip(new_clip(first_index, first_audio));
    let mut clip_ids: Vec<Id> = track.clips.keys().cloned().collect();
    for (index, samples) in audio {
        let clip = project_state
            .add_clip_to_audio_track(&track.id, new_clip(index, samples))
            .unwrap();
        clip_ids.push(clip.id);
    }

    let data_nodes = DataNodes::build(&project_state, &asset_pool, || false).unwrap();
    (data_nodes, clip_ids)
}

fn clip_event(clip_id: &Id, start_sample: usize, end_sample: usize) -> SchedulerEvent {
    SchedulerEvent::new(
        clip_id.clone(),
        SchedulerNodeKind::Clip,
        start_sample,
        end_sample,
    )
}

fn track_with(events: Vec<SchedulerEvent>) -> SchedulerAudioTrack {
    SchedulerAudioTrack {
        id: "track".to_string(),
        name: "track".to_string(),
        volume: 1.0,
        pan: 0.0,
        muted: false,
        clips: events,
    }
}

fn render(
    track: &SchedulerAudioTrack,
    data_nodes: &DataNodes,
    position_samples: usize,
    len: usize,
) -> Vec<f32> {
    // Start from garbage to prove that rendering leaves no stale data behind.
    let mut out = vec![9.0; len];
    track.render(position_samples, &mut out, data_nodes);
    out
}

#[test]
fn a_clip_inside_the_buffer_is_placed_at_its_start() {
    let (nodes, ids) = data_nodes_with_clips(vec![vec![1.0; 4]]);
    let track = track_with(vec![clip_event(&ids[0], 2, 6)]);
    assert_eq!(
        render(&track, &nodes, 0, 8),
        [0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0]
    );
}

#[test]
fn two_adjacent_clips_in_one_buffer_are_both_rendered_in_place() {
    let (nodes, ids) = data_nodes_with_clips(vec![vec![1.0; 6], vec![2.0; 6]]);
    let track = track_with(vec![clip_event(&ids[0], 0, 6), clip_event(&ids[1], 6, 12)]);
    assert_eq!(
        render(&track, &nodes, 0, 8),
        [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0]
    );
}

#[test]
fn a_buffer_spanning_the_end_of_one_clip_and_the_start_of_the_next() {
    let (nodes, ids) = data_nodes_with_clips(vec![vec![1.0; 6], vec![2.0; 6]]);
    let track = track_with(vec![clip_event(&ids[0], 0, 6), clip_event(&ids[1], 6, 12)]);
    assert_eq!(
        render(&track, &nodes, 4, 8),
        [1.0, 1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]
    );
}

#[test]
fn a_gap_between_two_clips_in_one_buffer_is_silent() {
    let (nodes, ids) = data_nodes_with_clips(vec![vec![1.0; 4], vec![2.0; 4]]);
    let track = track_with(vec![clip_event(&ids[0], 0, 4), clip_event(&ids[1], 6, 10)]);
    assert_eq!(
        render(&track, &nodes, 0, 8),
        [1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 2.0]
    );
}

#[test]
fn a_clip_played_over_consecutive_buffers_is_continuous() {
    let ramp: Vec<f32> = (1..=16).map(|n| n as f32).collect();
    let (nodes, ids) = data_nodes_with_clips(vec![ramp.clone()]);
    let track = track_with(vec![clip_event(&ids[0], 4, 20)]);

    let played: Vec<f32> = [0, 8, 16]
        .into_iter()
        .flat_map(|position| render(&track, &nodes, position, 8))
        .collect();

    let mut expected = vec![0.0; 4];
    expected.extend(ramp);
    expected.extend([0.0; 4]);
    assert_eq!(played, expected);
}

#[test]
fn buffers_that_only_touch_a_clip_are_silent() {
    let (nodes, ids) = data_nodes_with_clips(vec![vec![1.0; 4]]);
    let track = track_with(vec![clip_event(&ids[0], 8, 12)]);
    assert_eq!(render(&track, &nodes, 0, 8), [0.0; 8]);
    assert_eq!(render(&track, &nodes, 12, 8), [0.0; 8]);
}

#[test]
fn an_event_whose_node_is_missing_is_silent() {
    let (nodes, _ids) = data_nodes_with_clips(vec![vec![1.0; 4]]);
    let track = track_with(vec![clip_event(&"no such clip".to_string(), 0, 4)]);
    assert_eq!(render(&track, &nodes, 0, 8), [0.0; 8]);
}
