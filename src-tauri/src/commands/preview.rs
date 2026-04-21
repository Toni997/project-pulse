use crate::audio::preview_mixer::PREVIEW_MIXER;

#[tauri::command]
pub fn preview_play(file_path: String) {
    PREVIEW_MIXER.play(&file_path);
}
