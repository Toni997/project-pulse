use crate::{
    audio::{
        clip::{Clip, ClipToInsert},
        project_state::PROJECT_STATE,
        track::{AudioTrack, SamplerTrack},
    },
    core::types::Id,
};

#[tauri::command]
pub fn mixer_add_audio_track() -> AudioTrack {
    PROJECT_STATE.add_audio_track()
}

#[tauri::command]
pub async fn mixer_add_sampler_track(source_path: Option<String>) -> Option<SamplerTrack> {
    PROJECT_STATE.add_sampler_track(source_path).await
}

#[tauri::command]
pub async fn mixer_add_clip_to_audio_track(clip: ClipToInsert) -> Option<Clip> {
    PROJECT_STATE.add_clip_to_audio_track(clip).await
}

#[tauri::command]
pub async fn mixer_add_audio_track_with_clip(clip: ClipToInsert) -> Option<AudioTrack> {
    PROJECT_STATE.add_audio_track_with_clip(clip).await
}

// #[tauri::command]
// pub async fn mixer_add_instrument_track(source_path: Option<String>) -> Option<SamplerTrack> {
//     PROJECT_STATE.add_instrument_track(source_path).await
// }

#[tauri::command]
pub async fn mixer_assign_source_to_sampler_track(
    track_id: Id,
    source_path: Option<String>,
) -> Option<Id> {
    PROJECT_STATE
        .assign_source_to_sampler_track(track_id, source_path)
        .await
}

#[tauri::command]
pub fn mixer_move_clip_in_audio_track(track_id: Id, clip_id: Id, start_ppq: usize) -> Option<Clip> {
    PROJECT_STATE.move_clip_in_audio_track(track_id, clip_id, start_ppq)
}
