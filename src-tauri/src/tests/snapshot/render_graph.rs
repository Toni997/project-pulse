use std::sync::mpsc;

use super::*;

fn project_with_tracks(count: usize) -> (ProjectState, Vec<Id>) {
    let (changes, _receiver) = mpsc::channel();
    let project_state = ProjectState::new(changes);
    let ids = (0..count)
        .map(|_| project_state.add_audio_track().id.clone())
        .collect();
    (project_state, ids)
}

#[test]
fn a_new_graph_already_contains_the_master_node() {
    let graph = RenderGraph::new();
    let master = graph.get_node_by_id(MASTER_NODE_ID).unwrap();
    assert_eq!(master.data_node_id(), MASTER_NODE_ID);
    assert_eq!(graph.graph.node_count(), 1);
}

#[test]
fn every_track_gets_a_node_feeding_the_master() {
    let (project_state, track_ids) = project_with_tracks(2);
    let graph = RenderGraph::build(&project_state, || false).unwrap();

    assert_eq!(graph.graph.node_count(), 3);
    for track_id in &track_ids {
        let index = graph.id_to_index_map[track_id];
        assert_eq!(
            graph.get_node_by_index(index).unwrap().data_node_id(),
            track_id
        );
        assert!(graph.graph.contains_edge(index, graph.master_node));
    }
}

#[test]
fn nodes_are_found_by_id_only_when_they_exist() {
    let (project_state, track_ids) = project_with_tracks(1);
    let graph = RenderGraph::build(&project_state, || false).unwrap();

    assert!(graph.get_node_by_id(&track_ids[0]).is_some());
    assert!(graph.get_node_by_id("no such node").is_none());
}

#[test]
fn an_aborted_build_produces_nothing() {
    let (project_state, _) = project_with_tracks(2);
    assert!(RenderGraph::build(&project_state, || true).is_none());
}

#[test]
fn a_build_stops_when_an_edit_lands_while_it_runs() {
    let (project_state, _) = project_with_tracks(2);

    let version_at_start = project_state.version();
    let checks = std::cell::Cell::new(0);
    let result = RenderGraph::build(&project_state, || {
        checks.set(checks.get() + 1);
        if checks.get() == 2 {
            // An edit arrives part way through the build.
            project_state.add_audio_track();
        }
        project_state.version() != version_at_start
    });

    assert!(result.is_none());
}
