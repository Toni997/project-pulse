use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, LazyLock,
};

use arc_swap::ArcSwap;
use log::info;
use nanoid::nanoid;
use tauri::async_runtime;

use crate::audio::snapshot::data_nodes::DataNodes;
use crate::audio::snapshot::render_graph::RenderGraph;
use crate::audio::snapshot::scheduler::Scheduler;
use crate::core::constants::{PPQ_DEFAULT, TEMPO_BPM_DEFAULT};
use crate::core::types::Id;

#[derive(Clone)]
pub struct ProjectSnapshot {
    pub version: Id,
    pub scheduler_version: Id,
    pub render_graph_version: Id,
    pub data_nodes_version: Id,
    pub ppq: u16,
    pub tempo_bpm: f32,
    pub scheduler: Arc<Scheduler>,
    pub render_graph: Arc<RenderGraph>,
    pub data_nodes: Arc<DataNodes>,
}

impl ProjectSnapshot {
    pub fn new() -> Self {
        ProjectSnapshot {
            version: nanoid!(),
            scheduler_version: nanoid!(),
            render_graph_version: nanoid!(),
            data_nodes_version: nanoid!(),
            ppq: PPQ_DEFAULT,
            tempo_bpm: TEMPO_BPM_DEFAULT,
            scheduler: Arc::new(Scheduler::new()),
            render_graph: Arc::new(RenderGraph::new()),
            data_nodes: Arc::new(DataNodes::new()),
        }
    }

    pub fn get_scheduler(&self) -> &Scheduler {
        self.scheduler.as_ref()
    }

    pub fn get_data_nodes(&self) -> &DataNodes {
        self.data_nodes.as_ref()
    }

    pub fn with_ppq(&self, ppq: u16) -> Self {
        Self {
            version: nanoid!(),
            ppq,
            ..self.clone()
        }
    }

    pub fn with_tempo_bpm(&self, tempo_bpm: f32) -> Self {
        Self {
            version: nanoid!(),
            tempo_bpm,
            ..self.clone()
        }
    }

    pub fn with_scheduler(&self, scheduler: Arc<Scheduler>, scheduler_version: Id) -> Self {
        Self {
            version: nanoid!(),
            scheduler_version,
            scheduler,
            ..self.clone()
        }
    }

    pub fn with_render_graph(
        &self,
        render_graph: Arc<RenderGraph>,
        render_graph_version: Id,
    ) -> Self {
        Self {
            version: nanoid!(),
            render_graph_version,
            render_graph,
            ..self.clone()
        }
    }

    pub fn with_data_nodes(&self, data_nodes: Arc<DataNodes>, data_nodes_version: Id) -> Self {
        Self {
            version: nanoid!(),
            data_nodes_version,
            data_nodes,
            ..self.clone()
        }
    }
}

pub static PROJECT_SNAPSHOT: LazyLock<ArcSwap<ProjectSnapshot>> =
    LazyLock::new(|| ArcSwap::from_pointee(ProjectSnapshot::new()));

static SCHEDULER_REBUILD_GEN: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static RENDER_GRAPH_REBUILD_GEN: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));
static DATA_NODES_REBUILD_GEN: LazyLock<AtomicU64> = LazyLock::new(|| AtomicU64::new(0));

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
    PROJECT_SNAPSHOT
        .rcu(move |current| Arc::new(current.with_scheduler(Arc::clone(&scheduler), nanoid!())));
}

pub fn set_project_render_graph(render_graph: Arc<RenderGraph>) {
    PROJECT_SNAPSHOT.rcu(move |current| {
        Arc::new(current.with_render_graph(Arc::clone(&render_graph), nanoid!()))
    });
}

pub fn set_project_data_nodes(data_nodes: Arc<DataNodes>) {
    PROJECT_SNAPSHOT
        .rcu(move |current| Arc::new(current.with_data_nodes(Arc::clone(&data_nodes), nanoid!())));
}

pub fn rebuild_scheduler() {
    async_runtime::spawn_blocking(move || {
        info!("Rebuilding scheduler...");
        let gen = SCHEDULER_REBUILD_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let scheduler_version = nanoid!();

        let scheduler = Scheduler::build(|| SCHEDULER_REBUILD_GEN.load(Ordering::SeqCst) != gen);

        let Some(scheduler) = scheduler else {
            return;
        };

        if SCHEDULER_REBUILD_GEN.load(Ordering::SeqCst) != gen {
            return;
        }

        let scheduler = Arc::new(scheduler);
        PROJECT_SNAPSHOT.rcu(move |current| {
            Arc::new(current.with_scheduler(Arc::clone(&scheduler), scheduler_version.clone()))
        });
    });
}

pub fn rebuild_render_graph() {
    async_runtime::spawn_blocking(move || {
        info!("Rebuilding render graph...");
        let gen = RENDER_GRAPH_REBUILD_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let render_graph_version = nanoid!();

        let render_graph =
            RenderGraph::build(|| RENDER_GRAPH_REBUILD_GEN.load(Ordering::SeqCst) != gen);

        let Some(render_graph) = render_graph else {
            return;
        };

        if RENDER_GRAPH_REBUILD_GEN.load(Ordering::SeqCst) != gen {
            return;
        }

        let render_graph = Arc::new(render_graph);
        PROJECT_SNAPSHOT.rcu(move |current| {
            Arc::new(
                current.with_render_graph(Arc::clone(&render_graph), render_graph_version.clone()),
            )
        });
    });
}

pub fn rebuild_data_nodes() {
    async_runtime::spawn_blocking(move || {
        info!("Rebuilding data nodes...");
        let gen = DATA_NODES_REBUILD_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        let data_nodes_version = nanoid!();

        let data_nodes = DataNodes::build(|| DATA_NODES_REBUILD_GEN.load(Ordering::SeqCst) != gen);

        let Some(data_nodes) = data_nodes else {
            return;
        };

        if DATA_NODES_REBUILD_GEN.load(Ordering::SeqCst) != gen {
            return;
        }

        let data_nodes = Arc::new(data_nodes);
        PROJECT_SNAPSHOT.rcu(move |current| {
            Arc::new(current.with_data_nodes(Arc::clone(&data_nodes), data_nodes_version.clone()))
        });
    });
}
