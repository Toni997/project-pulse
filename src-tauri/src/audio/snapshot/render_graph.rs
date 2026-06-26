use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nanoid::nanoid;
use petgraph::{graph::NodeIndex, Graph};

use crate::audio::project_state::PROJECT_STATE;
use crate::audio::snapshot::project_snapshot::ProjectSnapshot;
use crate::core::types::{EngineSampleFormat, Id};

pub struct GraphNode {
    data_node_id: Id,
}

impl GraphNode {
    pub fn new(data_node_id: Id) -> Self {
        Self { data_node_id }
    }
}

pub struct RenderGraph {
    version: Id,
    graph: Graph<GraphNode, ()>,
    id_to_index_map: HashMap<Id, NodeIndex>,
}

impl RenderGraph {
    const MASTER_NODE_ID: &'static str = "master";

    pub fn new() -> Self {
        Self {
            version: nanoid!(),
            graph: Graph::new(),
            id_to_index_map: HashMap::new(),
        }
    }

    pub fn get_node_by_index(&self, index: NodeIndex) -> Option<&GraphNode> {
        self.graph.node_weight(index)
    }

    pub fn get_node_by_id(&self, id: &Id) -> Option<&GraphNode> {
        self.id_to_index_map
            .get(id)
            .and_then(|index| self.graph.node_weight(*index))
    }

    pub fn build(should_abort: impl Fn() -> bool) -> Option<Self> {
        if should_abort() {
            return None;
        }

        let mut new_render_graph = RenderGraph::new();
        let aborted = AtomicBool::new(false);

        let master_node = new_render_graph
            .graph
            .add_node(GraphNode::new(Self::MASTER_NODE_ID.to_string()));
        new_render_graph
            .id_to_index_map
            .insert(Self::MASTER_NODE_ID.to_string(), master_node);

        PROJECT_STATE.with_tracks(|tracks| {
            for (track_id, _) in tracks.iter() {
                if should_abort() {
                    aborted.store(true, Ordering::SeqCst);
                    return;
                }

                let track_node = new_render_graph
                    .graph
                    .add_node(GraphNode::new(track_id.clone()));
                new_render_graph
                    .id_to_index_map
                    .insert(track_id.clone(), track_node);

                new_render_graph.graph.add_edge(track_node, master_node, ());
            }
        });

        if aborted.load(Ordering::SeqCst) {
            return None;
        }

        Some(new_render_graph)
    }
}
