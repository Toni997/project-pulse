use std::sync::{mpsc, Arc};

use crate::engine::asset_pool::AssetPool;
use crate::engine::device::AudioEngine;
use crate::engine::mixing_thread_pool::{max_worker_count, MixingThreadPool};
use crate::engine::preview_mixer::PreviewMixer;
use crate::engine::transport::Transport;
use crate::project::project_state::ProjectState;
use crate::services::project_editor::ProjectEditor;
use crate::snapshot::publisher::SnapshotPublisher;

/// Owns every piece of engine/project state for the app. Constructed once in
/// `setup()` and handed to Tauri via `app.manage(...)`; commands reach it
/// through `tauri::State<Arc<AppState>>` instead of naming a global.
///
/// This type is deliberately just wiring: it constructs each subsystem and
/// hands out `Arc`s. Behavior (mutation logic, rebuild orchestration, engine
/// internals) lives on the subsystems themselves - `ProjectState`,
/// `SnapshotPublisher`, `Transport`, etc. - not here.
pub struct AppState {
    pub project_state: Arc<ProjectState>,
    pub editor: Arc<ProjectEditor>,
    pub asset_pool: Arc<AssetPool>,
    pub audio_engine: Arc<AudioEngine>,
    pub mixing_thread_pool: Arc<MixingThreadPool>,
    pub preview_mixer: Arc<PreviewMixer>,
    pub transport: Arc<Transport>,
    pub snapshot: Arc<SnapshotPublisher>,
}

impl AppState {
    /// Builds every subsystem, loads settings/project, and starts the audio
    /// output. The ring buffers are created here so each producer goes
    /// straight to the subsystem that owns it and the consumers straight into
    /// the output stream.
    pub fn bootstrap(_loaded_file_path: Option<&str>) -> Arc<Self> {
        let asset_pool = Arc::new(AssetPool::new());
        let audio_engine = Arc::new(AudioEngine::new());
        let mixing_thread_pool = Arc::new(MixingThreadPool::new(max_worker_count()));
        let (producers, consumers) = audio_engine.create_ring_buffers();
        let preview_mixer = Arc::new(PreviewMixer::new(
            Arc::clone(&audio_engine),
            producers.preview,
        ));

        let (change_tx, change_rx) = mpsc::channel();
        let project_state = Arc::new(ProjectState::new(change_tx));

        let editor = Arc::new(ProjectEditor::new(
            Arc::clone(&project_state),
            Arc::clone(&asset_pool),
            Arc::clone(&audio_engine),
        ));

        let snapshot = SnapshotPublisher::new(
            Arc::clone(&project_state),
            Arc::clone(&asset_pool),
            Arc::clone(&audio_engine),
            change_rx,
        );

        let transport = Arc::new(Transport::new(
            Arc::clone(&audio_engine),
            Arc::clone(&preview_mixer),
            Arc::clone(&mixing_thread_pool),
            Arc::clone(&snapshot),
            producers.engine,
        ));

        // load settings
        // load project
        audio_engine.start(consumers);

        Arc::new(Self {
            project_state,
            editor,
            asset_pool,
            audio_engine,
            mixing_thread_pool,
            preview_mixer,
            transport,
            snapshot,
        })
    }
}
