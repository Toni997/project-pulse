use crate::audio::timeline_mixer::TIMELINE_MIXER;

#[tauri::command(rename = "timeline::new_audio_track")]
pub fn timeline_add_audio_track() {
    TIMELINE_MIXER.add_audio_track();
}
