use crate::audio::{project_state::PROJECT_STATE, transport::TRANSPORT};

#[tauri::command]
pub fn transport_stop() {
    PROJECT_STATE.stop();
}

#[tauri::command]
pub fn transport_play() {
    TRANSPORT.play();
}
