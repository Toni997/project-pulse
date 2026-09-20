use std::sync::{mpsc::Receiver, Arc};
use std::time::Duration;

use arc_swap::ArcSwap;
use log::info;

use crate::core::rebuild_scope::RebuildScope;
use crate::engine::asset_pool::AssetPool;
use crate::engine::device::AudioEngine;
use crate::project::project_state::ProjectState;
use crate::snapshot::data_nodes::DataNodes;
use crate::snapshot::project_snapshot::ProjectSnapshot;
use crate::snapshot::render_graph::RenderGraph;
use crate::snapshot::scheduler::Scheduler;

/// How long the project must stay unedited before a build that an edit
/// interrupted is tried again.
const SETTLE_DELAY: Duration = Duration::from_millis(10);

/// Owns the live `ProjectSnapshot` and keeps it up to date with the project.
///
/// A single thread does all the building, so there is never more than
/// one build in flight no matter how many edits arrive. Requests from edits
/// are merged into one set of "parts still to rebuild". A build reads the
/// project without blocking edits, and gives up as soon as the project's
/// version changes, since what it read may be out of date. The parts it was
/// building stay in the set and are rebuilt, with any newer requests, once
/// edits have paused.
pub struct SnapshotPublisher {
    snapshot: ArcSwap<ProjectSnapshot>,
    project_state: Arc<ProjectState>,
    asset_pool: Arc<AssetPool>,
    audio_engine: Arc<AudioEngine>,
}

impl SnapshotPublisher {
    pub fn new(
        project_state: Arc<ProjectState>,
        asset_pool: Arc<AssetPool>,
        audio_engine: Arc<AudioEngine>,
        requests: Receiver<RebuildScope>,
    ) -> Arc<Self> {
        let publisher = Arc::new(Self {
            snapshot: ArcSwap::from_pointee(ProjectSnapshot::new()),
            project_state,
            asset_pool,
            audio_engine,
        });
        publisher.spawn_thread(requests);
        publisher
    }

    pub fn load(&self) -> Arc<ProjectSnapshot> {
        self.snapshot.load_full()
    }

    /// Holds only a `Weak` so the thread doesn't keep the publisher alive; it
    /// exits when the project state (the sender) is gone.
    fn spawn_thread(self: &Arc<Self>, requests: Receiver<RebuildScope>) {
        let publisher = Arc::downgrade(self);
        std::thread::spawn(move || {
            let mut pending = RebuildScope::empty();
            loop {
                if pending.is_empty() {
                    match requests.recv() {
                        Ok(scope) => pending |= scope,
                        Err(_) => return,
                    }
                }
                while let Ok(scope) = requests.try_recv() {
                    pending |= scope;
                }

                let Some(publisher) = publisher.upgrade() else {
                    return;
                };
                if publisher.rebuild(pending) {
                    pending = RebuildScope::empty();
                    continue;
                }

                // An edit interrupted the build: what it was rebuilding stays
                // pending. Wait for the edits to pause, folding in whatever
                // they ask for, then go again on the newer project.
                info!("Snapshot rebuild interrupted by an edit; retrying once edits settle");
                while let Ok(scope) = requests.recv_timeout(SETTLE_DELAY) {
                    pending |= scope;
                }
            }
        });
    }

    /// Rebuilds the requested parts and publishes them in one step, keeping the
    /// other parts as they are. Returns `false`, having published nothing, if
    /// an edit changed the project while it was building.
    fn rebuild(&self, scope: RebuildScope) -> bool {
        info!("Rebuilding snapshot ({scope:?})...");

        let version = self.project_state.version();
        let interrupted = || self.project_state.version() != version;

        let scheduler = if scope.contains(RebuildScope::SCHEDULER) {
            let Some(scheduler) = Scheduler::build(
                &self.project_state,
                &self.asset_pool,
                &self.audio_engine,
                interrupted,
            ) else {
                return false;
            };
            Some(Arc::new(scheduler))
        } else {
            None
        };

        let render_graph = if scope.contains(RebuildScope::RENDER_GRAPH) {
            let Some(render_graph) = RenderGraph::build(&self.project_state, interrupted) else {
                return false;
            };
            Some(Arc::new(render_graph))
        } else {
            None
        };

        let data_nodes = if scope.contains(RebuildScope::DATA_NODES) {
            let Some(data_nodes) =
                DataNodes::build(&self.project_state, &self.asset_pool, interrupted)
            else {
                return false;
            };
            Some(Arc::new(data_nodes))
        } else {
            None
        };

        // An edit that landed after the last check inside a build would make
        // the results stale too.
        if interrupted() {
            return false;
        }

        // This thread is the only writer, so nothing can change the snapshot
        // between loading it and storing the new one.
        let current = self.snapshot.load();
        let next = current.with_parts(
            scheduler.unwrap_or_else(|| Arc::clone(current.scheduler())),
            render_graph.unwrap_or_else(|| Arc::clone(current.render_graph())),
            data_nodes.unwrap_or_else(|| Arc::clone(current.data_nodes())),
        );
        self.snapshot.store(Arc::new(next));
        true
    }
}
