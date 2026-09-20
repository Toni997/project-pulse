use std::collections::HashMap;

use derive_more::TryInto;
use indexmap::IndexMap;
use log::warn;

use crate::{
    core::{constants::MASTER_NODE_ID, types::Id},
    engine::asset_pool::AssetPool,
    project::clip::Clip,
    project::project_state::ProjectState,
    snapshot::{
        bus_node::BusNode, clip_node::ClipNode, master_node::MasterNode, track_node::TrackNode,
    },
};

#[derive(TryInto)]
#[try_into(ref)]
pub enum DataNode {
    ClipNode(ClipNode),
    TrackNode(TrackNode),
    BusNode(BusNode),
    MasterNode(MasterNode),
}

pub struct DataNodes {
    nodes: HashMap<Id, DataNode>,
}

impl DataNodes {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Looks up a node by id, as the node type `T`. `None` if there is no such
    /// node, or if it is a different kind of node.
    pub fn get_node<'a, T>(&'a self, id: &str) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a DataNode>,
    {
        self.nodes
            .get(id)
            .and_then(|node| <&T>::try_from(node).ok())
    }

    /// Builds the nodes one track at a time, holding the project's lock only
    /// while reading a single track, and gives up (`None`) as soon as
    /// `should_abort` says the project has changed underneath it.
    pub fn build(
        project_state: &ProjectState,
        asset_pool: &AssetPool,
        should_abort: impl Fn() -> bool,
    ) -> Option<Self> {
        if should_abort() {
            return None;
        }

        let mut new_data_nodes = DataNodes::new();

        let master = project_state
            .with_master(|master| MasterNode::new(master.volume, master.pan, master.muted));
        new_data_nodes
            .nodes
            .insert(MASTER_NODE_ID.to_string(), DataNode::MasterNode(master));

        for track_id in project_state.track_ids() {
            if should_abort() {
                return None;
            }

            // `None` from `with_track` means the track was deleted since its id
            // was listed, i.e. an edit happened; `None` from inside means abort.
            project_state
                .with_track(&track_id, |track| {
                    let track = track.common();
                    new_data_nodes.nodes.insert(
                        track_id.clone(),
                        DataNode::TrackNode(TrackNode::new(track.volume, track.pan, track.muted)),
                    );
                    new_data_nodes.add_clip_nodes(&track.clips, asset_pool, &should_abort)
                })
                .flatten()?;
        }

        for bus_id in project_state.bus_ids() {
            if should_abort() {
                return None;
            }

            project_state
                .with_bus(&bus_id, |bus| {
                    new_data_nodes.nodes.insert(
                        bus_id.clone(),
                        DataNode::BusNode(BusNode::new(bus.volume, bus.pan, bus.muted)),
                    );
                    new_data_nodes.add_clip_nodes(&bus.clips, asset_pool, &should_abort)
                })
                .flatten()?;
        }

        Some(new_data_nodes)
    }

    /// Adds a node for each clip whose audio is loaded. `None` if the build
    /// was aborted part way through.
    fn add_clip_nodes(
        &mut self,
        clips: &IndexMap<Id, Clip>,
        asset_pool: &AssetPool,
        should_abort: &impl Fn() -> bool,
    ) -> Option<()> {
        for (clip_id, clip) in clips {
            if should_abort() {
                return None;
            }

            if let Some(clip_node) = Self::build_clip_node(clip, asset_pool) {
                self.nodes
                    .insert(clip_id.clone(), DataNode::ClipNode(clip_node));
            }
        }
        Some(())
    }

    /// `None` (with a warning) when the clip's audio isn't loaded; such a clip
    /// simply has no node and stays silent.
    fn build_clip_node(clip: &Clip, asset_pool: &AssetPool) -> Option<ClipNode> {
        let Some(samples) = asset_pool.audio.get_samples_by_id(&clip.source_id) else {
            warn!(
                "DataNodes: no audio loaded for clip {} (source {})",
                clip.id, clip.source_id
            );
            return None;
        };
        Some(ClipNode::new(samples, clip.source_offset_samples))
    }
}

#[cfg(test)]
#[path = "../tests/snapshot/data_nodes.rs"]
mod tests;
