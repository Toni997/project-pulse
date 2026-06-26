use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use petgraph::visit::Data;

use crate::audio::clip::Clip;
use crate::{
    audio::project_state::PROJECT_STATE,
    audio::snapshot::{
        bus_node::BusNode, clip_node::ClipNode, master_node::MasterNode, track_node::TrackNode,
    },
    audio::track::GeneratorTrack,
    core::types::Id,
};

pub enum DataNode {
    ClipNode(ClipNode),
    TrackNode(TrackNode),
    BusNode(BusNode),
    MasterNode(MasterNode),
}

impl DataNode {
    pub fn as_clip(&self) -> Option<&ClipNode> {
        match self {
            DataNode::ClipNode(clip) => Some(clip),
            _ => None,
        }
    }
}

pub struct DataNodes {
    pub nodes: HashMap<Id, DataNode>,
}

impl DataNodes {
    const MASTER_NODE_ID: &'static str = "master";

    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn build(should_abort: impl Fn() -> bool) -> Option<Self> {
        if should_abort() {
            return None;
        }

        let mut new_data_nodes = DataNodes::new();
        let aborted = AtomicBool::new(false);

        PROJECT_STATE.with_master(|master| {
            new_data_nodes.nodes.insert(
                Self::MASTER_NODE_ID.to_string(),
                DataNode::MasterNode(MasterNode {
                    volume: master.volume,
                    pan: master.pan,
                    muted: master.muted,
                }),
            );
        });

        PROJECT_STATE.with_tracks(|tracks| {
            for (track_id, track) in tracks.iter() {
                if should_abort() {
                    aborted.store(true, Ordering::SeqCst);
                    return;
                }

                let (volume, pan, muted, clips) = match track {
                    GeneratorTrack::AudioTrack(t) => (t.volume, t.pan, t.muted, &t.clips),
                    GeneratorTrack::SamplerTrack(t) => (t.volume, t.pan, t.muted, &t.clips),
                };

                new_data_nodes.nodes.insert(
                    track_id.clone(),
                    DataNode::TrackNode(TrackNode { volume, pan, muted }),
                );

                for (clip_id, clip) in clips.iter() {
                    if should_abort() {
                        aborted.store(true, Ordering::SeqCst);
                        return;
                    }

                    new_data_nodes.nodes.insert(
                        clip_id.clone(),
                        DataNode::ClipNode(ClipNode {
                            source_id: clip.source_id.clone(),
                            source_offset_samples: clip.source_offset_samples,
                        }),
                    );
                }
            }
        });

        if aborted.load(Ordering::SeqCst) {
            return None;
        }

        PROJECT_STATE.with_buses(|buses| {
            for (bus_id, bus) in buses.iter() {
                if should_abort() {
                    aborted.store(true, Ordering::SeqCst);
                    return;
                }

                new_data_nodes.nodes.insert(
                    bus_id.clone(),
                    DataNode::BusNode(BusNode {
                        volume: bus.volume,
                        pan: bus.pan,
                        muted: bus.muted,
                    }),
                );

                for (clip_id, clip) in bus.clips.iter() {
                    if should_abort() {
                        aborted.store(true, Ordering::SeqCst);
                        return;
                    }

                    new_data_nodes.nodes.insert(
                        clip_id.clone(),
                        DataNode::ClipNode(ClipNode {
                            source_id: clip.source_id.clone(),
                            source_offset_samples: clip.source_offset_samples,
                        }),
                    );
                }
            }
        });

        if aborted.load(Ordering::SeqCst) {
            return None;
        }

        Some(new_data_nodes)
    }
}
