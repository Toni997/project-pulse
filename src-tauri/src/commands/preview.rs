use tauri::AppHandle;

use crate::audio::preview_mixer::PREVIEW_MIXER;

#[tauri::command]
pub fn preview_play(app: AppHandle, file_path: String) {
    PREVIEW_MIXER.play(app, &file_path);
}
