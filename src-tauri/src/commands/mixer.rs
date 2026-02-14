use crate::audio::project_state::PROJECT_STATE;

#[tauri::command]
pub fn mixer_add_audio_track(source_path: Option<String>) {
    PROJECT_STATE.add_audio_track(source_path);
}
