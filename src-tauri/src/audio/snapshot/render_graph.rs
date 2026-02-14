use petgraph::{graph::NodeIndex, Graph};

use crate::audio::snapshot::{
    bus_node::BusNode, clip_node::ClipNode, master_node::MasterNode, track_node::TrackNode,
};

pub enum RenderNode {
    ClipNode(ClipNode),
    TrackNode(TrackNode),
    BusNode(BusNode),
    MasterNode(MasterNode),
}

pub trait Render {
    fn render(&self) -> Vec<f32>;
}

pub struct RenderGraph {
    graph: Graph<RenderNode, ()>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&RenderNode> {
        self.graph.node_weight(index)
    }
}
