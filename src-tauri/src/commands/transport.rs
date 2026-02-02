use crate::audio::timeline_mixer::TIMELINE_MIXER;

#[tauri::command(rename = "transport::stop")]
pub fn transport_stop() {
    TIMELINE_MIXER.stop();
}
