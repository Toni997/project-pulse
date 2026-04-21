use std::sync::{Arc, LazyLock};

use arc_swap::ArcSwap;

use crate::audio::snapshot::render_graph::RenderGraph;
use crate::audio::snapshot::scheduler::Scheduler;
use crate::audio::snapshot::transport_runtime::TransportRuntime;
use crate::core::constants::{PPQ_DEFAULT, TEMPO_BPM_DEFAULT};

#[derive(Clone)]
pub struct ProjectSnapshot {
    pub ppq: u16,
    pub tempo_bpm: f32,
    pub scheduler: Arc<Scheduler>,
    pub render_graph: Arc<RenderGraph>,
    pub transport_runtime: Arc<TransportRuntime>,
}

impl ProjectSnapshot {
    pub fn new() -> Self {
        ProjectSnapshot {
            ppq: PPQ_DEFAULT,
            tempo_bpm: TEMPO_BPM_DEFAULT,
            scheduler: Arc::new(Scheduler::new()),
            render_graph: Arc::new(RenderGraph::new()),
            transport_runtime: Arc::new(TransportRuntime::new()),
        }
    }

    pub fn with_ppq(&self, ppq: u16) -> Self {
        Self {
            ppq,
            ..self.clone()
        }
    }

    pub fn with_tempo_bpm(&self, tempo_bpm: f32) -> Self {
        Self {
            tempo_bpm,
            ..self.clone()
        }
    }

    pub fn with_scheduler(&self, scheduler: Arc<Scheduler>) -> Self {
        Self {
            scheduler,
            ..self.clone()
        }
    }

    pub fn with_render_graph(&self, render_graph: Arc<RenderGraph>) -> Self {
        Self {
            render_graph,
            ..self.clone()
        }
    }

    pub fn with_transport_runtime(&self, transport_runtime: Arc<TransportRuntime>) -> Self {
        Self {
            transport_runtime,
            ..self.clone()
        }
    }
}

pub static PROJECT_SNAPSHOT: LazyLock<ArcSwap<ProjectSnapshot>> =
    LazyLock::new(|| ArcSwap::from_pointee(ProjectSnapshot::new()));

pub fn load_project_snapshot() -> Arc<ProjectSnapshot> {
    PROJECT_SNAPSHOT.load_full()
}

pub fn replace_project_snapshot(snapshot: ProjectSnapshot) {
    PROJECT_SNAPSHOT.store(Arc::new(snapshot));
}

pub fn set_project_ppq(ppq: u16) {
    PROJECT_SNAPSHOT.rcu(|current| Arc::new(current.with_ppq(ppq)));
}

pub fn set_project_tempo_bpm(tempo_bpm: f32) {
    PROJECT_SNAPSHOT.rcu(|current| Arc::new(current.with_tempo_bpm(tempo_bpm)));
}

pub fn set_project_scheduler(scheduler: Arc<Scheduler>) {
    PROJECT_SNAPSHOT.rcu(move |current| Arc::new(current.with_scheduler(Arc::clone(&scheduler))));
}

pub fn set_project_render_graph(render_graph: Arc<RenderGraph>) {
    PROJECT_SNAPSHOT
        .rcu(move |current| Arc::new(current.with_render_graph(Arc::clone(&render_graph))));
}

pub fn set_project_transport_runtime(transport_runtime: Arc<TransportRuntime>) {
    PROJECT_SNAPSHOT.rcu(move |current| {
        Arc::new(current.with_transport_runtime(Arc::clone(&transport_runtime)))
    });
}
