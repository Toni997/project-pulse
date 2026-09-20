use std::sync::Arc;

use nanoid::nanoid;

use crate::core::constants::{PPQ_DEFAULT, TEMPO_BPM_DEFAULT};
use crate::core::types::Id;
use crate::snapshot::data_nodes::DataNodes;
use crate::snapshot::render_graph::RenderGraph;
use crate::snapshot::scheduler::Scheduler;

/// An immutable view of the project prepared for playback. A snapshot is
/// never modified once published; a change produces a new snapshot instead.
#[derive(Clone)]
pub struct ProjectSnapshot {
    version: Id,
    ppq: u16,
    tempo_bpm: f32,
    scheduler: Arc<Scheduler>,
    render_graph: Arc<RenderGraph>,
    data_nodes: Arc<DataNodes>,
}

impl ProjectSnapshot {
    pub fn new() -> Self {
        ProjectSnapshot {
            version: nanoid!(),
            ppq: PPQ_DEFAULT,
            tempo_bpm: TEMPO_BPM_DEFAULT,
            scheduler: Arc::new(Scheduler::new()),
            render_graph: Arc::new(RenderGraph::new()),
            data_nodes: Arc::new(DataNodes::new()),
        }
    }

    pub fn version(&self) -> &Id {
        &self.version
    }

    pub fn scheduler(&self) -> &Arc<Scheduler> {
        &self.scheduler
    }

    pub fn render_graph(&self) -> &Arc<RenderGraph> {
        &self.render_graph
    }

    pub fn data_nodes(&self) -> &Arc<DataNodes> {
        &self.data_nodes
    }

    /// Replaces one or more parts at once, bumping the shared version. Used
    /// by `SnapshotPublisher::rebuild` to publish whatever it actually
    /// recomputed - parts it didn't touch are passed through unchanged by
    /// the caller (see there for why a single version still works even
    /// though parts can now be rebuilt independently).
    pub fn with_parts(
        &self,
        scheduler: Arc<Scheduler>,
        render_graph: Arc<RenderGraph>,
        data_nodes: Arc<DataNodes>,
    ) -> Self {
        Self {
            version: nanoid!(),
            scheduler,
            render_graph,
            data_nodes,
            ..self.clone()
        }
    }
}
