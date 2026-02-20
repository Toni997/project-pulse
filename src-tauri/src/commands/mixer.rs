use crate::{
    audio::{project_state::PROJECT_STATE, track::AudioTrack},
    core::types::Id,
};

#[tauri::command]
pub async fn mixer_add_audio_track(source_path: Option<String>) -> Option<AudioTrack> {
    PROJECT_STATE.add_audio_track(source_path).await
}

#[tauri::command]
pub async fn mixer_assign_audio_to_track(track_id: Id, source_path: Option<String>) -> Option<Id> {
    PROJECT_STATE
        .assign_audio_to_track(track_id, source_path)
        .await
}
