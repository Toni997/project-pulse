use std::sync::Arc;

use tauri::State;

use crate::app_state::AppState;

#[tauri::command]
pub fn preview_play(state: State<Arc<AppState>>, file_path: String) {
    state.preview_mixer.play(&file_path);
}
