use std::collections::HashMap;

use petgraph::{graph::NodeIndex, Graph};

use crate::core::constants::MASTER_NODE_ID;
use crate::core::types::Id;
use crate::project::project_state::ProjectState;

pub struct GraphNode {
    data_node_id: Id,
}

impl GraphNode {
    pub fn new(data_node_id: Id) -> Self {
        Self { data_node_id }
    }

    /// Id of the data node this graph node stands for.
    pub fn data_node_id(&self) -> &Id {
        &self.data_node_id
    }
}

pub struct RenderGraph {
    graph: Graph<GraphNode, ()>,
    id_to_index_map: HashMap<Id, NodeIndex>,
    master_node: NodeIndex,
}

impl RenderGraph {
    /// A graph containing just the master node.
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let master_node = graph.add_node(GraphNode::new(MASTER_NODE_ID.to_string()));
        let mut id_to_index_map = HashMap::new();
        id_to_index_map.insert(MASTER_NODE_ID.to_string(), master_node);

        Self {
            graph,
            id_to_index_map,
            master_node,
        }
    }

    pub fn get_node_by_index(&self, index: NodeIndex) -> Option<&GraphNode> {
        self.graph.node_weight(index)
    }

    pub fn get_node_by_id(&self, id: &str) -> Option<&GraphNode> {
        self.id_to_index_map
            .get(id)
            .and_then(|index| self.graph.node_weight(*index))
    }

    pub fn build(project_state: &ProjectState, should_abort: impl Fn() -> bool) -> Option<Self> {
        if should_abort() {
            return None;
        }

        let mut new_render_graph = RenderGraph::new();
        for track_id in project_state.track_ids() {
            if should_abort() {
                return None;
            }

            let track_node = new_render_graph
                .graph
                .add_node(GraphNode::new(track_id.clone()));
            new_render_graph
                .id_to_index_map
                .insert(track_id, track_node);
            new_render_graph
                .graph
                .add_edge(track_node, new_render_graph.master_node, ());
        }

        Some(new_render_graph)
    }
}

#[cfg(test)]
#[path = "../tests/snapshot/render_graph.rs"]
mod tests;
