use std::sync::Arc;

use tauri::State;

use crate::{
    app_state::AppState,
    core::types::Id,
    project::{
        clip::{Clip, ClipToInsert},
        track::{AudioTrack, SamplerTrack},
    },
};

#[tauri::command]
pub fn mixer_add_audio_track(state: State<Arc<AppState>>) -> AudioTrack {
    state.project_state.add_audio_track()
}

#[tauri::command]
pub async fn mixer_add_sampler_track(
    state: State<'_, Arc<AppState>>,
    source_path: Option<String>,
) -> Result<Option<SamplerTrack>, String> {
    Ok(state.editor.add_sampler_track(source_path).await)
}

#[tauri::command]
pub async fn mixer_add_clip_to_audio_track(
    state: State<'_, Arc<AppState>>,
    clip: ClipToInsert,
) -> Result<Option<Clip>, String> {
    Ok(state.editor.add_clip_to_audio_track(clip).await)
}

#[tauri::command]
pub async fn mixer_add_audio_track_with_clip(
    state: State<'_, Arc<AppState>>,
    clip: ClipToInsert,
) -> Result<Option<AudioTrack>, String> {
    Ok(state.editor.add_audio_track_with_clip(clip).await)
}

// #[tauri::command]
// pub async fn mixer_add_instrument_track(source_path: Option<String>) -> Option<SamplerTrack> {
//     PROJECT_STATE.add_instrument_track(source_path).await
// }

#[tauri::command]
pub async fn mixer_assign_source_to_sampler_track(
    state: State<'_, Arc<AppState>>,
    track_id: Id,
    source_path: Option<String>,
) -> Result<Option<Id>, String> {
    Ok(state
        .editor
        .assign_source_to_sampler_track(track_id, source_path)
        .await)
}

#[tauri::command]
pub fn mixer_move_clip_in_audio_track(
    state: State<Arc<AppState>>,
    track_id: Id,
    clip_id: Id,
    start_ppq: usize,
) -> Option<Clip> {
    state
        .project_state
        .move_clip_in_audio_track(&track_id, &clip_id, start_ppq)
}

#[tauri::command]
pub fn mixer_delete_clip_from_audio_track(state: State<Arc<AppState>>, track_id: Id, clip_id: Id) {
    state
        .project_state
        .delete_clip_from_audio_track(&track_id, &clip_id);
}
