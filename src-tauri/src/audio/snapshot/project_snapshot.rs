use std::sync::LazyLock;

use crate::audio::snapshot::render_graph::RenderGraph;
use crate::audio::snapshot::scheduler::Scheduler;
use crate::audio::snapshot::transport_runtime::TransportRuntime;
use crate::core::constants::{PPQ_DEFAULT, TEMPO_BPM_DEFAULT};

pub struct ProjectSnapshot {
    pub ppq: u16,
    pub tempo_bpm: f32,
    pub scheduler: Scheduler,
    pub render_graph: RenderGraph,
    pub transport_runtime: TransportRuntime,
}

impl ProjectSnapshot {
    pub fn new() -> Self {
        ProjectSnapshot {
            ppq: PPQ_DEFAULT,
            tempo_bpm: TEMPO_BPM_DEFAULT,
            scheduler: Scheduler::new(),
            render_graph: RenderGraph::new(),
            transport_runtime: TransportRuntime::new(),
        }
    }
}

pub static PROJECT_SNAPSHOT: LazyLock<ProjectSnapshot> = LazyLock::new(|| ProjectSnapshot::new());
