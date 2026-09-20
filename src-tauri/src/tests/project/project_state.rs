use std::sync::mpsc::{self, Receiver};

use super::*;

fn project() -> (ProjectState, Receiver<RebuildScope>) {
    let (requests, receiver) = mpsc::channel();
    (ProjectState::new(requests), receiver)
}

fn clip() -> NewClip {
    NewClip {
        name: "clip".to_string(),
        start_ppq: 0,
        length_ppq: 10,
        source_id: "source".to_string(),
    }
}

#[test]
fn every_successful_edit_bumps_the_version() {
    let (project, _requests) = project();
    assert_eq!(project.version(), 0);

    let track = project.add_audio_track();
    assert_eq!(project.version(), 1);

    let clip = project.add_clip_to_audio_track(&track.id, clip()).unwrap();
    assert_eq!(project.version(), 2);

    project.move_clip_in_audio_track(&track.id, &clip.id, 5);
    assert_eq!(project.version(), 3);

    project.delete_clip_from_audio_track(&track.id, &clip.id);
    assert_eq!(project.version(), 4);

    project.delete_audio_track(&track.id);
    assert_eq!(project.version(), 5);
}

#[test]
fn edits_that_change_nothing_leave_the_version_alone() {
    let (project, requests) = project();
    let track = project.add_audio_track();
    let version = project.version();
    while requests.try_recv().is_ok() {}

    assert!(project.add_clip_to_audio_track("nope", clip()).is_err());
    assert!(project
        .move_clip_in_audio_track(&track.id, "nope", 1)
        .is_none());
    project.delete_clip_from_audio_track(&track.id, "nope");
    project.delete_audio_track("nope");

    assert_eq!(project.version(), version);
    assert!(requests.try_recv().is_err());
}

#[test]
fn each_edit_asks_for_only_the_parts_it_invalidated() {
    let (project, requests) = project();

    let track = project.add_audio_track();
    assert_eq!(
        requests.try_recv().unwrap(),
        RebuildScope::RENDER_GRAPH | RebuildScope::DATA_NODES
    );

    let clip = project.add_clip_to_audio_track(&track.id, clip()).unwrap();
    assert_eq!(
        requests.try_recv().unwrap(),
        RebuildScope::DATA_NODES | RebuildScope::SCHEDULER
    );

    project.move_clip_in_audio_track(&track.id, &clip.id, 5);
    assert_eq!(requests.try_recv().unwrap(), RebuildScope::SCHEDULER);
}

#[test]
fn tracks_are_listed_in_creation_order_and_found_by_id() {
    let (project, _requests) = project();
    let first = project.add_audio_track().id.clone();
    let second = project.add_audio_track().id.clone();

    assert_eq!(project.track_ids(), [first.clone(), second.clone()]);
    assert_eq!(
        project.with_track(&second, |track| track.id().to_string()),
        Some(second)
    );
    assert!(project.with_track("nope", |_| ()).is_none());
    assert!(project.with_bus("nope", |_| ()).is_none());
    assert!(project.bus_ids().is_empty());
}
